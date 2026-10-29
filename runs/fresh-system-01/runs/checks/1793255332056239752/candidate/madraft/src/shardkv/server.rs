use super::{msg::*,key2shard};
use crate::kvraft::{server::{Server, State},client::{Command,CLIENT_ID}};
use crate::shard_ctrler::{client::Clerk as CtrlerClerk,msg::Config,N_SHARDS};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap,net::SocketAddr,sync::{Arc,atomic::Ordering}};
use madsim::{task,time::{sleep,Duration}};
pub struct ShardKvServer {inner:Arc<Server<ShardKv>>}
impl ShardKvServer {
    pub async fn new(ctrl_ck:CtrlerClerk,servers:Vec<SocketAddr>,gid:u64,me:usize,max_raft_state:Option<usize>)->Arc<Self> {
        let inner=Server::<ShardKv>::new(servers,me,max_raft_state).await;
        inner.initialize(|s|s.gid=gid);
        let net=inner.endpoint();let owner=inner.clone();
        net.add_rpc_handler(move |req:Fetch| {let owner=owner.clone();async move {
            if req.shard>=N_SHARDS {return None;}
            if req.delete {
                if propose(&owner,Op::Delete{config:req.config,shard:req.shard}).await {Some(Transfer::default())}else{None}
            }else{owner.inspect(|s|s.exports.get(&(req.config,req.shard)).cloned())}
        }});
        let owner=inner.clone();
        task::spawn(async move {loop {
            sleep(Duration::from_millis(70)).await;
            if !owner.is_leader(){continue;}
            let (number,ready)=owner.inspect(|s|(s.config.num,s.status.iter().all(|x|*x<2)));
            if ready {
                if let Some(next)=number.checked_add(1) {
                    let config=ctrl_ck.query_at(next).await;
                    if config.num==next {propose(&owner,Op::Config{config}).await;}
                }
            }
            for shard in 0..N_SHARDS {
                let (number,status,donors)=owner.inspect(|s|(s.config.num,s.status[shard],s.previous.groups.get(&s.previous.shards[shard]).cloned().unwrap_or_default()));
                if status<2 {continue;}
                for donor in donors {
                    let response=net.call_timeout(donor,Fetch{config:number,shard,delete:status==3},Duration::from_millis(180)).await;
                    if let Ok(Some(data))=response {
                        if status==2 {propose(&owner,Op::Install{config:number,shard,data}).await;}
                        else {propose(&owner,Op::Ready{config:number,shard}).await;}
                        break;
                    }
                }
            }
        }});
        Arc::new(Self{inner})
    }
}
async fn propose(server:&Server<ShardKv>,args:Op)->bool {
    let id=CLIENT_ID.fetch_update(Ordering::SeqCst,Ordering::SeqCst,|v|v.checked_add(1)).expect("client IDs exhausted");
    server.apply(Command{client:id,seq:1,args}).await.is_ok()
}
#[derive(Debug,Default,Serialize,Deserialize)]
pub struct ShardKv {gid:u64,config:Config,previous:Config,status:[u8;N_SHARDS],shards:[Transfer;N_SHARDS],exports:HashMap<(u64,usize),Transfer>}
impl State for ShardKv {
    type Command=Op;type Output=Reply;
    fn use_sessions()->bool {false}
    fn apply(&mut self,cmd:Op)->Reply {self.apply_request(0,0,cmd)}
    fn apply_request(&mut self,client:u64,seq:u64,cmd:Op)->Reply {
        match cmd {
            Op::Config{config}=>{
                if self.config.num.checked_add(1)==Some(config.num) && self.status.iter().all(|x|*x<2) {
                    for shard in 0..N_SHARDS {
                        let old=self.config.shards[shard];let new=config.shards[shard];
                        if old==self.gid && new!=self.gid {self.exports.insert((config.num,shard),std::mem::take(&mut self.shards[shard]));self.status[shard]=0;}
                        else if old!=self.gid && new==self.gid {self.shards[shard]=Transfer::default();self.status[shard]=if old==0 {1}else{2};}
                    }
                    self.previous=std::mem::replace(&mut self.config,config);
                }
                Reply::Ok
            },
            Op::Install{config,shard,data}=>{if shard<N_SHARDS && config==self.config.num && self.status[shard]==2 {self.shards[shard]=data;self.status[shard]=3;}Reply::Ok},
            Op::Delete{config,shard}=>{if config<=self.config.num {self.exports.remove(&(config,shard));}Reply::Ok},
            Op::Ready{config,shard}=>{if shard<N_SHARDS && config==self.config.num && self.status[shard]==3 {self.status[shard]=1;}Reply::Ok},
            op=>{
                let key=match &op {Op::Get{key}|Op::Put{key,..}|Op::Append{key,..}=>key,_=>unreachable!()};
                let shard=key2shard(key);
                if self.config.shards[shard]!=self.gid || !matches!(self.status[shard],1|3) {return Reply::WrongGroup;}
                let data=&mut self.shards[shard];
                if let Some((old,value))=data.sessions.get(&client) {if *old>=seq {return value.clone();}}
                let result=match op {
                    Op::Get{key}=>Reply::Get{value:data.data.get(&key).cloned()},
                    Op::Put{key,value}=>{data.data.insert(key,value);Reply::Ok},
                    Op::Append{key,value}=>{data.data.entry(key).or_default().push_str(&value);Reply::Ok},_=>unreachable!()
                };
                data.sessions.insert(client,(seq,result.clone()));result
            }
        }
    }
}
