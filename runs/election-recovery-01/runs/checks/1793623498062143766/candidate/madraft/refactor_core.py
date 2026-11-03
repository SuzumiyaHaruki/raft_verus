p='madraft/src/raft/vote_core.rs';s=open(p).read();s=s.replace('pub struct Core {','pub struct Data {',1).replace('impl Core {','impl Data {',1).replace('    #[verifier::type_invariant]\n','',1).replace('Some(Core{','Some(Data{')
s=s.replace('use_type_invariant(&*self);','assert(self.wf());').replace('use_type_invariant(self);','assert(self.wf());')
# extract all named functions (fn) with braced bodies skipping braces inside contracts by body detection line indentation
import re
names=['domain','new','recover','inspect','receive','start_election','complete','collect','try_become_leader','vote_count','has_vote','demote']
starts=[]
for name in names:
 m=re.search(r'    (?:pub )?(?:proof )?fn '+name+r'\(',s);starts.append((m.start(),name))
starts.sort()
wrappers=[]
for idx,(a,name) in reversed(list(enumerate(starts))):
 end=starts[idx+1][0] if idx+1<len(starts) else s.index('\n}\nfn with_vote',a)
 segment=s[a:end]
 # body begins at first standalone indentation brace or inline brace after ensures for domain/inspect
 match=re.search(r'\n    \{',segment)
 assert match,(name,segment)
 b=match.start(); header=segment[:b]; body=segment[b:]
 orig_header=header
 mut='&mut self' in header
 isstatic=name in ['new','recover']
 if isstatic:
  header=header.replace('        ensures ', '        ensures out.is_some() ==> out.unwrap().wf(),\n            ',1)
 elif mut:
  header=header.replace('        ensures ', '        requires old(self).wf(),\n        ensures self.wf(),\n            ',1) if 'requires ' not in header else header.replace('requires term>=','requires old(self).wf(), term>=').replace('        ensures ','        ensures self.wf(),\n            ',1)
 else:
  if 'ensures ' in header: header=header.replace('ensures ','requires self.wf(), ensures ',1)
 s=s[:a]+header+body+s[end:]
 if name=='demote':continue
 # wrapper header original interfaces and body delegate
 args={'domain':'','new':'c,initial','recover':'c,record','inspect':'','receive':'request','start_election':'','complete':'notice','collect':'context,result','try_become_leader':'','vote_count':'','has_vote':'member'}[name]
 if isstatic:
  wb='{ let d=Data::'+name+'('+args+'); match d {Some(data)=>Some(Core{data}),None=>None} }'
 else:
  wb='{ '+('use_type_invariant(self);' if name=='domain' else 'proof { use_type_invariant(&*self); }')+' self.data.'+name+'('+args+') }'
 wrappers.append((name,orig_header+'\n    '+wb+'\n'))
# insert outer core and methods
pos=s.index('pub struct Data')
s=s[:pos]+'''pub struct Core { data: Data }
impl Core {
    #[verifier::type_invariant]
    closed spec fn wf(&self) -> bool { self.data.wf() }
    pub closed spec fn view(&self) -> StateView { self.data.view() }
    pub closed spec fn choices(&self) -> Seq<Choice> { self.data.choices() }
    pub closed spec fn base(&self) -> Option<Record> { self.data.base() }
    pub closed spec fn votes(&self) -> Set<u64> { self.data.votes() }
'''+''.join(x[1] for x in reversed(wrappers))+'}\n'+s[pos:]
# Data constructors use Self, already Data initializers
# inner methods must no_unwind so Core mut field call permitted
for name in ['receive','start_election','complete','collect','try_become_leader']:
 # only Data method (last occurrence), insert no_unwind before body header end
 start=s.rfind('pub fn '+name+'(');end=s.index('\n    {',start);s=s[:end]+'\n        no_unwind'+s[end:]
# field helper demote cannot keep pending at term change, requires pending none
s=s.replace('requires old(self).wf(), term>=old(self).state.state.term,','requires old(self).wf(), old(self).state.pending.is_none(), term>=old(self).state.state.term,')
open(p,'w').write(s)
