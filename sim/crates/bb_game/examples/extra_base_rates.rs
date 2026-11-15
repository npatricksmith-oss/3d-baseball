//! Diagnostic (from the Codex runner-reads review probe): how often a runner takes the extra base
//! on a hit, by the official scorer's ruling: first to third on a single, second to home on a
//! single, first to home on a double, over games from seed 0, with the tags the lab counts.
//! `-- [games]`
use bb_game::{game::{Game,classify_bip},play::FielderAttrs};
use bb_rules::{Event,Input,OutKind};
use bb_stats::scorer::{score_play,Outcome,DEFAULT_CHOICE_MARGIN_S};
fn main(){
 let a:Vec<String>=std::env::args().collect(); let n:u64=a.get(1).map(|x|x.parse().unwrap()).unwrap_or(100);
 let mut counts=[[0usize;2];3]; let mut outs=[0usize;3]; let mut hits=0; let mut warnings=0;
 for seed in 0..n {
  let mut g=Game::new(seed,false).with_fielders(core::array::from_fn(FielderAttrs::average));
  let mut data=Vec::new();
  loop {
   let (bases,outs_before)=(g.st.bases,g.st.outs);
   let Some(s)=g.step_pitch() else {break};
   warnings+=s.rules_events.iter().filter(|e|matches!(e,Event::RulesWarning{..})).count();
   let (reached,hit)=classify_bip(s.batter,&s.rules_events,&s.facts);
   if reached {
    if hit {hits+=1}
    for e in &s.rules_events {if let Event::RunnerOut{runner,kind:OutKind::Tag,..}=e {outs[if hit {if *runner==s.batter {0}else{1}}else{2}]+=1}}
   }
   data.push((s.play,bases,outs_before,s.facts));
  }
  let res=g.finish(); let log=bb_schema::log::read(&res.log).unwrap();
  for (ix,bases,ob,facts) in data {
   let Some(sc)=score_play(log.plays[ix as usize],ob as u32,DEFAULT_CHOICE_MARGIN_S) else {continue};
   let Outcome::Hit(hb)=sc.outcome else {continue};
   for (i,start,end,hitbase) in [(0,1,3,1),(1,2,4,1),(2,1,4,2)] {
    if hb!=hitbase {continue}
    if let Some(id)=bases[start-1] {
     counts[i][0]+=1;
     if facts.iter().any(|f|matches!(f,Input::BaseTouched{agent,base,holding_ball:false,..} if *agent==id && *base>=end)){counts[i][1]+=1}
    }
   }
  }
 }
 println!("games {n}, rates [opportunities, taken] {counts:?}, lab hit plays {hits}, tags batter/other/FC {outs:?}, warnings {warnings}");
}
