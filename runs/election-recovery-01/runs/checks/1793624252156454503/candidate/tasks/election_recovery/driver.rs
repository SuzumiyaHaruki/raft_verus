    /// Trigger one campaign through the real handle; future timers may call this.
    /// Each actual RPC result is consumed by this same Core; the Core decides promotion after each completed operation.
    pub async fn start_election(&self) -> Result<Vec<(u64, std::result::Result<(u64, bool), String>)>> {
        let serial = self.vote_serial.lock().await;
        let decision = self.vote_call(|c| c.start_election());
        let write = match decision {
            vote_api::Campaign::Save(w) => w,
            vote_api::Campaign::Reject(f) => return Err(io::Error::other(format!("campaign: {f:?}")).into()),
        };
        let hooks = self.vote_hooks.lock().unwrap().clone();
        let saved = vote_host::save(write, &hooks).await;
        let action = self.vote_call(|c| c.complete(vote_api::Completion{
            op:write.record.op,success:saved.is_ok()}));
        self.vote_call(|c| c.try_become_leader());
        saved?;
        let Some(vote_api::Action::Send(send)) = action else {
            return Err(io::Error::other("core did not authorize campaign send").into());
        };
        drop(serial);
        self.send_vote_request(send).await
    }

    // UNVERIFIED thin adapter: use the immutable authorized payload verbatim.
    // It never reads another term/vote/role or reconstructs a later-round request.
    async fn send_vote_request(&self, send: vote_api::Send)
        -> Result<Vec<(u64, std::result::Result<(u64, bool), String>)>>
    {
        let peers = self.inner.lock().unwrap().peers.clone();
        if send.config.members != peers.len() as u64 || send.config.node >= send.config.members {
            return Err(io::Error::other("authorized membership cannot be mapped to peers").into());
        }
        let mut rpcs = FuturesUnordered::new();
        for node in 0..send.config.members {
            if node == send.config.node { continue; }
            let net = self.endpoint.clone();
            let peer = peers[node as usize];
            let r = send.request;
            let args = RequestVoteArgs{term:r.term,candidate_id:r.candidate,
                last_log_term:r.log.term,last_log_index:r.log.index};
            let hook=self.vote_hooks.lock().unwrap().reply_barrier.clone();
            rpcs.push(async move {
                let response = net.call_timeout::<RequestVoteArgs>(peer,args,Duration::from_secs(2)).await;
                let response = match response {
                    Ok(Ok(r)) => Ok((r.term,r.vote_granted)),
                    Ok(Err(e)) => Err(e), Err(e) => Err(e.to_string()),
                };
                let context=vote_api::RpcContext{sent:send,peer:node};
                let raw=match &response {
                    Ok((term,granted)) => vote_api::RpcResult::Reply{term:*term,granted:*granted},
                    Err(_) => vote_api::RpcResult::Error,
                };
                let delayed=async {
                    if let Some(h)=hook {
                        if h.peer==context.peer && h.term==context.sent.request.term {
                            h.entered.send((context,raw)).await.map_err(|e|io::Error::other(e.to_string()))?;
                            h.release.recv().await.map_err(|e|io::Error::other(e.to_string()))?;
                        }
                    }
                    Ok::<(),io::Error>(())
                }.await;
                (context,response,raw,delayed)
            });
        }
        let mut observations = Vec::new();let mut failure=None;
        while let Some((context,result,raw,delayed)) = rpcs.next().await {
            delayed?;
            if let Err(e)=self.deliver_vote_result(context,raw).await {
                if failure.is_none() { failure=Some(e); }
            }
            observations.push((context.peer,result));
        }
        if let Some(error)=failure { return Err(error); }
        Ok(observations)
    }

    // UNVERIFIED wiring: immutable original context and actual result, no vote rules.
    async fn deliver_vote_result(&self, context: vote_api::RpcContext, result: vote_api::RpcResult) -> Result<()> {
        let _serial = self.vote_serial.lock().await;
        let out = self.vote_call(|c| c.collect(context,result));
        match out {
            vote_api::Collect::Save(write) => {
                let hooks = self.vote_hooks.lock().unwrap().clone();
                let saved = vote_host::save(write,&hooks).await;
                let action = self.vote_call(|c| c.complete(vote_api::Completion{
                    op:write.record.op,success:saved.is_ok()}));
                self.vote_call(|c| c.try_become_leader());
                saved?;
                if action != Some(vote_api::Action::Persisted) {
                    return Err(io::Error::other("reply-term save produced an unexpected protocol action").into());
                }
            },
            vote_api::Collect::Counted | vote_api::Collect::Ignored => {},
            vote_api::Collect::Reject(f) => return Err(io::Error::other(format!("collection: {f:?}")).into()),
        }
        self.vote_call(|c| c.try_become_leader());
        let observer = self.vote_hooks.lock().unwrap().collected.clone();
        if let Some(sender) = observer { let _ = sender.try_send((context,result)); }
        Ok(())
    }

    pub fn vote_count(&self) -> u64 { self.inner.lock().unwrap().vote.vote_count() }
    pub fn has_vote(&self, member:u64) -> bool { self.inner.lock().unwrap().vote.has_vote(member) }

    // Passive observation only: the closure contains the actual protocol call.
    // Callers hold vote_serial across each operation, never across network waits.
    fn vote_call<R>(&self, f:impl FnOnce(&mut vote_core::Core)->R) -> R {
        let mut raft=self.inner.lock().unwrap();
        let before=raft.vote.inspect();let node=raft.me as u64;
        let result=f(&mut raft.vote);let after=raft.vote.inspect();
        if let Some(log)=&self.vote_hooks.lock().unwrap().changes {
            log.lock().unwrap().push(election_observation::Change{node,before,after});
        }
        result
    }
