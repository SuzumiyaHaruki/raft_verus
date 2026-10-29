use super::{msg::*,key2shard};
use crate::{shard_ctrler::client::Clerk as CtrlerClerk,kvraft::client::{Command,CLIENT_ID}};
use std::{net::SocketAddr,sync::atomic::{AtomicU64,Ordering}};
use madsim::{net,time::{sleep,Duration}};
pub struct Clerk {ctrl_ck:CtrlerClerk,id:u64,seq:AtomicU64,serial:futures::lock::Mutex<()>}
impl Clerk {
    pub fn new(servers:Vec<SocketAddr>)->Self {Self{ctrl_ck:CtrlerClerk::new(servers),id:CLIENT_ID.fetch_update(Ordering::SeqCst,Ordering::SeqCst,|v|v.checked_add(1)).expect("client IDs exhausted"),seq:AtomicU64::new(0),serial:futures::lock::Mutex::new(())}}
    async fn call(&self,key:&str,args:Op)->Reply {
        let _guard=self.serial.lock().await;
        let seq=self.seq.fetch_update(Ordering::SeqCst,Ordering::SeqCst,|v|v.checked_add(1)).expect("request IDs exhausted")+1;
        let cmd=Command{client:self.id,seq,args};let net=net::Endpoint::bind("0.0.0.0:0").await.unwrap();
        loop {
            let config=self.ctrl_ck.query().await;
            if let Some(servers)=config.groups.get(&config.shards[key2shard(key)]) {
                for peer in servers {
                    match net.call_timeout(*peer,cmd.clone(),Duration::from_millis(700)).await {
                        Ok(Ok(Reply::WrongGroup))=>break,
                        Ok(Ok(value))=>return value,_=>{}
                    }
                }
            }
            sleep(Duration::from_millis(70)).await;
        }
    }
    pub async fn get(&self,key:String)->String {match self.call(&key,Op::Get{key:key.clone()}).await {Reply::Get{value}=>value.unwrap_or_default(),_=>unreachable!()}}
    pub async fn put(&self,key:String,value:String){self.call(&key,Op::Put{key:key.clone(),value}).await;}
    pub async fn append(&self,key:String,value:String){self.call(&key,Op::Append{key:key.clone(),value}).await;}
}
