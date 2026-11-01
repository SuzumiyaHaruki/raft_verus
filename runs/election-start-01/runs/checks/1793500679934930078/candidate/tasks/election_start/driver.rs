    /// Trigger one campaign through the real handle; future timers may call this.
    /// Returned RPC results are observations only: no tally or leader promotion.
    pub async fn start_election(&self) -> Result<Vec<(u64, std::result::Result<(u64, bool), String>)>> {
        let serial = self.vote_serial.lock().await;
        let decision = self.inner.lock().unwrap().vote.start_election();
        let write = match decision {
            vote_api::Campaign::Save(w) => w,
            vote_api::Campaign::Reject(f) => return Err(io::Error::other(format!("campaign: {f:?}")).into()),
        };
        let hooks = self.vote_hooks.lock().unwrap().clone();
        let saved = vote_host::save(write, &hooks).await;
        let action = self.inner.lock().unwrap().vote.complete(vote_api::Completion{
            op:write.record.op,success:saved.is_ok()});
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
            rpcs.push(async move {
                let response = net.call_timeout::<RequestVoteArgs>(peer,args,Duration::from_secs(2)).await;
                let response = match response {
                    Ok(Ok(r)) => Ok((r.term,r.vote_granted)),
                    Ok(Err(e)) => Err(e), Err(e) => Err(e.to_string()),
                };
                (node,response)
            });
        }
        let mut observations = Vec::new();
        while let Some(r) = rpcs.next().await { observations.push(r); }
        Ok(observations)
    }
