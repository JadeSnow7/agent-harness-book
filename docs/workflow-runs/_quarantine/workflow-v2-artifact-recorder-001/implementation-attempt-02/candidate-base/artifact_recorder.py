#!/usr/bin/env python3
import argparse,hashlib,json,os,re
from pathlib import Path
def die(e,c=2): print(json.dumps({'ok':False,'error':e})); raise SystemExit(c)
def rootpath(root,rel):
 p=(root/rel).resolve(); r=root.resolve()
 if not str(p).startswith(str(r)+os.sep): die('path_escape')
 return p
def main():
 ap=argparse.ArgumentParser(); s=ap.add_subparsers(dest='cmd',required=True)
 for n in ('inspect','record'):
  q=s.add_parser(n); q.add_argument('--repo-root',default='.'); q.add_argument('--artifact-root'); q.add_argument('--descriptor'); q.add_argument('--source')
 q=s.add_parser('verify'); q.add_argument('--repo-root',default='.'); q.add_argument('--artifact-root',required=True); q.add_argument('--task-id'); q.add_argument('--chapter')
 a=ap.parse_args(); r=Path(a.repo_root).resolve()
 if a.cmd=='inspect': print(json.dumps({'ok':True,'command':'inspect'})); return
 if a.cmd=='record':
  d=json.loads(Path(a.descriptor).read_text()); src=Path(d['source_path']); b=src.read_bytes()
  if len(b)!=d['expected_byte_length'] or hashlib.sha256(b).hexdigest()!=d['expected_sha256']: die('source_integrity',3)
  t=rootpath(rootpath(r,a.artifact_root),d['target_path']); m=Path(re.sub(r'\.payload\.[^.]+$','.manifest.yaml',str(t)))
  if t.exists() or m.exists(): die('already_exists',3)
  t.parent.mkdir(parents=True,exist_ok=True); t.write_bytes(b)
  x=dict(d); x.update(byte_length=len(b),sha256=hashlib.sha256(b).hexdigest(),target_path=d['target_path'])
  m.write_text(json.dumps(x,ensure_ascii=False,sort_keys=True,indent=2)+'\n'); print(json.dumps({'ok':True,'sequence':d['sequence']})); return
 ar=rootpath(r,a.artifact_root); seq=[]
 for p in ar.glob('*.payload.*'):
  m=Path(re.sub(r'\.payload\.[^.]+$','.manifest.yaml',str(p)))
  if not m.exists(): die('manifest_missing',3)
  x=json.loads(m.read_text()); b=p.read_bytes()
  if x.get('byte_length')!=len(b) or x.get('sha256')!=hashlib.sha256(b).hexdigest(): die('integrity',3)
  seq.append(x.get('sequence'))
 seq=sorted(seq)
 if seq and seq!=list(range(1,max(seq)+1)): die('sequence',3)
 print(json.dumps({'ok':True,'artifact_count':len(seq),'last_sequence':max(seq,default=0)}))
if __name__=='__main__': main()
