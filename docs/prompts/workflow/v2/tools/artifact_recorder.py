#!/usr/bin/env python3
"""Fail-closed exact-byte artifact recorder."""
from __future__ import annotations
import argparse, hashlib, json, os, re, tempfile
from pathlib import Path

OK, INPUT, INTEGRITY, SECURITY = 0, 2, 3, 4
ROLES={"task-analysis","planning","implementation","post-implementation-audit","summary","commit","commit-audit","artifact-recorder","controller-state","controller-correction","user-gate-requirement","user-gate-implementation","approved-prompt","task-package","foundation-audit"}
STATUSES={"passed","failed","changes_required","needs_user_input","blocked","returned","rejected","correction_required","confirmed","approved","snapshot","awaiting_requirement_confirmation","planning","awaiting_implementation_approval","implementation","audit_failed","plaintext_unavailable","recorded"}
PROVENANCE={"platform_raw_export","bootstrap_handoff","manual_raw_export","platform_jsonl_encrypted_content"}
REQUIRED={"source_path","expected_byte_length","expected_sha256","task_id","chapter","sequence","artifact_key","role","attempt","status","payload_extension","media_type","target_path","source_provenance","source_reference","identity_source","historical","recorded_by","revision_of"}
OPTIONAL={"representation","plaintext_available","plaintext_verified","content_auditability","exception_reason","exception_approval_ref","claim_source","export_method","attestation","payload_header","external_descriptor","platform_metadata","claimed_role","claimed_status","required_fields","source_line_number"}
TARGET=re.compile(r"^(\d{3})-([a-z0-9]+(?:-[a-z0-9]+)*)-attempt-(\d{2})\.payload\.([a-z0-9]+)$")
FENCE=re.compile(r"\A---\n(.*?)\n---(?:\n|\Z)",re.S)
PLACEHOLDER=re.compile(r"(?:<[^>]+>|\b(?:YOUR|EXAMPLE|REDACTED|CHANGEME|PLACEHOLDER)[-_A-Z0-9]*\b)",re.I)
SECRETS=(re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),re.compile(r"Authorization\s*:\s*Bearer\s+([^\s<]+)",re.I),re.compile(r"\b(?:OPENAI|AWS|GH|GITHUB)_[A-Z0-9_]*\s*=\s*([^\s#<]+)"),re.compile(r"\b(?:sk|pk)-[A-Za-z0-9]{20,}\b"),re.compile(r"\bAKIA[0-9A-Z]{16}\b"),re.compile(r"\b(?:token|secret|api[_-]?key)\s*[:=]\s*([^\s<]{20,})",re.I))

class Failure(Exception):
    def __init__(self,code,field=None,location=None,category="input",rc=INPUT): self.code,self.field,self.location,self.category,self.rc=code,field,location,category,rc
def fail(*a,**kw): raise Failure(*a,**kw)
def out(e):
    value={"ok":False,"error":{"category":e.category,"code":e.code}}
    if e.field:value["error"]["field"]=e.field
    if e.location:value["error"]["location"]=e.location
    print(json.dumps(value,ensure_ascii=False,sort_keys=True));raise SystemExit(e.rc)
def digest(data): return hashlib.sha256(data).hexdigest()
def regular(path,label):
    if path.is_symlink() or not path.is_file(): fail("not_regular_file",location=label,category="security",rc=SECURITY)
def confined(root,relative,field):
    if not isinstance(relative,str) or not relative or os.path.isabs(relative): fail("unsafe_path",field,category="security",rc=SECURITY)
    p=Path(relative)
    if "\\" in relative or any(x in ("",".","..") for x in p.parts): fail("unsafe_path",field,category="security",rc=SECURITY)
    base=root.resolve();candidate=base.joinpath(*p.parts);cur=base
    for part in p.parts:
        cur/=part
        if cur.is_symlink(): fail("symlink_escape",field,category="security",rc=SECURITY)
    try:
        if os.path.commonpath((str(base),str(candidate.resolve())))!=str(base): fail("path_escape",field,category="security",rc=SECURITY)
    except ValueError: fail("path_escape",field,category="security",rc=SECURITY)
    return candidate
def descriptor(path):
    regular(path,"descriptor")
    try:o=json.loads(path.read_bytes().decode())
    except (OSError,UnicodeDecodeError,json.JSONDecodeError): fail("descriptor_invalid",field="descriptor")
    if not isinstance(o,dict): fail("descriptor_type",field="descriptor")
    if set(o)-REQUIRED-OPTIONAL: fail("unknown_descriptor_field",field=sorted(set(o)-REQUIRED-OPTIONAL)[0])
    if REQUIRED-set(o): fail("descriptor_missing",field=sorted(REQUIRED-set(o))[0])
    for k in ("expected_byte_length","sequence","attempt"):
        if not isinstance(o[k],int) or isinstance(o[k],bool) or o[k]<1: fail("descriptor_integer",field=k)
    for k in ("task_id","chapter","artifact_key","role","status","payload_extension","media_type","source_provenance","source_reference","identity_source","recorded_by"):
        if not isinstance(o[k],str) or not o[k]: fail("descriptor_string",field=k)
    if o["source_provenance"] not in PROVENANCE: fail("provenance_invalid",field="source_provenance")
    if not isinstance(o["historical"],bool) or (o["revision_of"] is not None and not isinstance(o["revision_of"],str)): fail("descriptor_type")
    if not re.fullmatch(r"[0-9a-f]{64}",o["expected_sha256"]): fail("sha256_format",field="expected_sha256")
    if o["role"] not in ROLES: fail("role_invalid",field="role")
    if o["status"] not in STATUSES: fail("status_invalid",field="status")
    m=TARGET.fullmatch(o["target_path"])
    if not m or int(m[1])!=o["sequence"] or m[2]!=o["role"] or int(m[3])!=o["attempt"] or m[4]!=o["payload_extension"]: fail("target_identity",field="target_path",category="integrity",rc=INTEGRITY)
    rep=o.get("representation","plaintext_utf8")
    if rep not in {"plaintext_utf8","encrypted_platform_record"}: fail("representation_invalid",field="representation")
    if o.get("export_method") in {"manual_copy","clipboard_copy","reconstructed_from_chat"}: fail("raw_export_required",field="export_method")
    if o.get("source_provenance")=="manual_raw_export" and not o.get("attestation"): fail("manual_attestation_required",field="attestation")
    if rep=="encrypted_platform_record":
        valid=o["role"]=="controller-correction" and o["sequence"] in (6,8) and o.get("plaintext_available") is False and o.get("plaintext_verified") is False and o.get("content_auditability")=="unavailable" and o["identity_source"]=="platform_metadata" and o["source_provenance"]=="platform_jsonl_encrypted_content" and all(isinstance(o.get(k),str) and o[k] for k in ("exception_reason","exception_approval_ref","claim_source","claimed_role","claimed_status")) and o["claimed_role"]==o["role"] and o["claimed_status"]==o["status"]
        if not valid: fail("encrypted_exception_metadata",category="integrity",rc=INTEGRITY)
    elif o.get("plaintext_verified") is False: fail("plaintext_flag_invalid",category="integrity",rc=INTEGRITY)
    return o
def sensitive(data):
    text=data.decode("utf-8",errors="ignore")
    return any(not PLACEHOLDER.search(m.group(0)) for p in SECRETS for m in p.finditer(text))
def source_bytes(o):
    p=Path(o["source_path"]);regular(p,"source_path");data=p.read_bytes();rep=o.get("representation","plaintext_utf8")
    try:text=data.decode()
    except UnicodeDecodeError:
        if rep=="plaintext_utf8": fail("invalid_utf8",field="source_path")
        fail("encrypted_record_not_jsonl",category="integrity",rc=INTEGRITY)
    if rep=="encrypted_platform_record":
        if "\n" in text or "\r" in text: fail("encrypted_record_not_single_line",category="integrity",rc=INTEGRITY)
        try:r=json.loads(text)
        except json.JSONDecodeError: fail("encrypted_record_not_jsonl",category="integrity",rc=INTEGRITY)
        if not isinstance(r,dict) or not isinstance(r.get("encrypted_content"),str) or not r["encrypted_content"] or "content" in r or not isinstance(r.get("platform_metadata"),dict) or not r.get("line_id") or not isinstance(r.get("sequence"),int) or not isinstance(r.get("role"),str) or not isinstance(r.get("status"),str): fail("encrypted_record_shape",category="integrity",rc=INTEGRITY)
    if sensitive(data): fail("sensitive_source",field="source_path",category="security",rc=SECURITY)
    if len(data)!=o["expected_byte_length"] or digest(data)!=o["expected_sha256"]: fail("source_integrity",category="integrity",rc=INTEGRITY)
    return data
def manifest_path(p): return p.with_name(re.sub(r"\.payload\.[^.]+$",".manifest.yaml",p.name))
def sync(p):
    fd=os.open(p,os.O_RDONLY)
    try:os.fsync(fd)
    finally:os.close(fd)
def install(path,data,stage):
    path.parent.mkdir(parents=True,exist_ok=True);fd,name=tempfile.mkstemp(prefix=f".{path.name}.",dir=str(path.parent));temp=Path(name)
    try:
        with os.fdopen(fd,"wb") as f:
            f.write(data);f.flush()
            if os.environ.get("ARTIFACT_RECORDER_FAILPOINT")==f"{stage}:file_fsync":raise OSError()
            os.fsync(f.fileno())
        if os.environ.get("ARTIFACT_RECORDER_FAILPOINT")==f"{stage}:link":raise OSError()
        os.link(temp,path);temp.unlink()
        if os.environ.get("ARTIFACT_RECORDER_FAILPOINT")==f"{stage}:dir_fsync":raise OSError()
        sync(path.parent)
    except FileExistsError:
        if temp.exists():temp.unlink()
        fail("already_exists",category="integrity",rc=INTEGRITY)
    except Exception:
        if temp.exists():temp.unlink()
        fail("install_failed",category="integrity",rc=INTEGRITY)
def inventory(root):
    if not root.exists():return [],[]
    if root.is_symlink() or not root.is_dir():fail("artifact_root_invalid",category="security",rc=SECURITY)
    ms=[];ps=[]
    for x in root.iterdir():
        if x.is_symlink():fail("symlink_artifact",location=x.name,category="security",rc=SECURITY)
        if x.is_dir():fail("unexpected_directory",location=x.name,category="integrity",rc=INTEGRITY)
        if x.name.startswith(".") or ".tmp" in x.name:fail("temporary_or_unknown",location=x.name,category="integrity",rc=INTEGRITY)
        if x.name.endswith(".manifest.yaml"):ms.append(x)
        elif ".payload." in x.name:ps.append(x)
        else:fail("unknown_artifact",location=x.name,category="integrity",rc=INTEGRITY)
    return ms,ps
def canonical(o):return (json.dumps(o,ensure_ascii=False,sort_keys=True,indent=2)+"\n").encode()
def manifests(paths):
    rows=[]
    for p in paths:
        try:raw=p.read_bytes();o=json.loads(raw.decode())
        except (OSError,UnicodeDecodeError,json.JSONDecodeError):fail("manifest_invalid",location=p.name,category="integrity",rc=INTEGRITY)
        if not isinstance(o,dict) or raw!=canonical(o):fail("manifest_not_canonical",location=p.name,category="integrity",rc=INTEGRITY)
        rows.append(o)
    return rows
def revisions(rows):
    prev={}
    for r in sorted(rows,key=lambda x:x.get("sequence",0)):
        old=prev.get(r.get("artifact_key"))
        if old is None:
            if r.get("attempt")!=1 or r.get("revision_of") is not None:fail("revision_root",category="integrity",rc=INTEGRITY)
        elif r.get("attempt")!=old.get("attempt",0)+1 or r.get("revision_of")!=old.get("manifest"):fail("revision",category="integrity",rc=INTEGRITY)
        prev[r.get("artifact_key")]=r
def verify_row(r,p,m,task,chapter):
    q=TARGET.fullmatch(p.name)
    if not q:fail("filename_invalid",location=p.name,category="integrity",rc=INTEGRITY)
    if r.get("task_id")!=task or r.get("chapter")!=chapter or r.get("sequence")!=int(q[1]) or r.get("role")!=q[2] or r.get("attempt")!=int(q[3]) or r.get("manifest")!=m.name:fail("identity_mismatch",location=p.name,category="integrity",rc=INTEGRITY)
    d=p.read_bytes()
    if r.get("byte_length")!=len(d) or r.get("sha256")!=digest(d):fail("payload_integrity",location=p.name,category="integrity",rc=INTEGRITY)
    text=m.read_text(errors="ignore")
    if "source_path" in r or "/tmp" in text or "descriptor_path" in text:fail("manifest_leak",location=m.name,category="security",rc=SECURITY)
    if r.get("representation")=="encrypted_platform_record":
        if r.get("plaintext_available") is not False or r.get("plaintext_verified") is not False:fail("encrypted_plaintext_claim",location=m.name,category="integrity",rc=INTEGRITY)
        try:x=json.loads(d.decode())
        except (UnicodeDecodeError,json.JSONDecodeError):fail("encrypted_record_invalid",location=p.name,category="integrity",rc=INTEGRITY)
        if x.get("sequence")!=r.get("sequence") or x.get("role")!=r.get("claimed_role") or x.get("status")!=r.get("claimed_status"):fail("encrypted_identity_mismatch",location=p.name,category="integrity",rc=INTEGRITY)
    else:
        h=FENCE.match(d.decode(errors="ignore"))
        if h:
            vals=dict(re.findall(r"(?m)^([a-z_]+):[ \t]*([^\n]+)$",h.group(1)))
            for k in ("task_id","role","status"):
                if k in vals and vals[k].strip(" '\"")!=str(r.get(k)):fail("payload_header_mismatch",field=k,category="integrity",rc=INTEGRITY)
def record(a):
    o=descriptor(Path(a.descriptor));d=source_bytes(o);root=confined(Path(a.repo_root).resolve(),a.artifact_root,"artifact_root");ms,ps=inventory(root);target=confined(root,o["target_path"],"target_path");manifest=manifest_path(target)
    orphan=False
    if len(ms)!=len(ps):
        if len(ps)==1 and not ms and ps[0]==target and ps[0].read_bytes()==d:orphan=True
        else:fail("pair_mismatch",category="integrity",rc=INTEGRITY)
    rows=manifests(ms)
    if o["sequence"]!=max([x.get("sequence",0) for x in rows],default=0)+1:fail("sequence",category="integrity",rc=INTEGRITY)
    same=sorted((x for x in rows if x.get("artifact_key")==o["artifact_key"]),key=lambda x:x.get("attempt",0))
    if same and (o["attempt"]!=same[-1].get("attempt",0)+1 or o.get("revision_of")!=same[-1].get("manifest")):fail("revision",category="integrity",rc=INTEGRITY)
    if not same and (o["attempt"]!=1 or o.get("revision_of") is not None):fail("revision_root",category="integrity",rc=INTEGRITY)
    if (target.exists() and not orphan) or manifest.exists():fail("already_exists",category="integrity",rc=INTEGRITY)
    meta=dict(o);meta.pop("source_path",None);meta.update(byte_length=len(d),sha256=digest(d),manifest=manifest.name,manifest_format="canonical-json-yaml",plaintext_verified=o.get("representation","plaintext_utf8")!="encrypted_platform_record")
    installed=[]
    try:
        if not orphan:install(target,d,"payload");installed.append(target)
        install(manifest,canonical(meta),"manifest");installed.append(manifest)
    except Exception:
        cleanup=None
        candidates = installed + [target, manifest]
        seen = set()
        for p in reversed(candidates):
            if p in seen:
                continue
            seen.add(p)
            try:
                if p.exists() and not p.is_symlink():p.unlink()
            except Exception as e:cleanup=e
        try:sync(root)
        except Exception as e:cleanup=cleanup or e
        if cleanup:fail("install_failed_cleanup_failed",category="integrity",rc=INTEGRITY)
        raise
    print(json.dumps({"ok":True,"command":"record","sequence":o["sequence"],"sha256":digest(d)},sort_keys=True))
def verify(a):
    root=confined(Path(a.repo_root).resolve(),a.artifact_root,"artifact_root");ms,ps=inventory(root)
    if len(ms)!=len(ps):fail("pair_mismatch",category="integrity",rc=INTEGRITY)
    rows=[]
    for p in ps:
        m=manifest_path(p)
        if not m.exists() or m.is_symlink():fail("manifest_missing",location=p.name,category="integrity",rc=INTEGRITY)
        r=manifests([m])[0];verify_row(r,p,m,a.task_id,a.chapter);rows.append(r)
    rows.sort(key=lambda x:x.get("sequence",0))
    if [x.get("sequence") for x in rows]!=list(range(1,len(rows)+1)):fail("sequence",category="integrity",rc=INTEGRITY)
    revisions(rows);print(json.dumps({"ok":True,"command":"verify","artifact_count":len(rows),"last_sequence":rows[-1]["sequence"] if rows else 0},sort_keys=True))
def inspect(a):
    p=Path(a.source);regular(p,"source");d=p.read_bytes();print(json.dumps({"ok":True,"command":"inspect","byte_length":len(d),"sha256":digest(d)},sort_keys=True))
def legacy(a):
    root=Path(a.artifact_root)
    if root.is_symlink() or not root.is_dir():fail("legacy_root_invalid",category="security",rc=SECURITY)
    files=[]
    for p in sorted(root.rglob("*")):
        if p.is_symlink():fail("legacy_symlink",location=str(p),category="security",rc=SECURITY)
        if p.is_file():
            d=p.read_bytes();files.append({"path":str(p.relative_to(root)),"nonempty":bool(d),"byte_length":len(d),"sha256":digest(d),"schema_differences":["legacy_unverified","manifest_schema_unverified"]})
    print(json.dumps({"ok":True,"command":"legacy-inspect","status":"legacy_unverified","artifact_count":len(files),"files":files},ensure_ascii=False,sort_keys=True))
def main():
    p=argparse.ArgumentParser(description=__doc__);s=p.add_subparsers(dest="command",required=True)
    x=s.add_parser("inspect");x.add_argument("--source",required=True)
    x=s.add_parser("legacy-inspect");x.add_argument("--artifact-root",required=True)
    x=s.add_parser("record");x.add_argument("--repo-root",default=".");x.add_argument("--artifact-root",required=True);x.add_argument("--descriptor",required=True)
    x=s.add_parser("verify");x.add_argument("--repo-root",default=".");x.add_argument("--artifact-root",required=True);x.add_argument("--task-id",required=True);x.add_argument("--chapter",required=True)
    a=p.parse_args()
    try:{"inspect":inspect,"legacy-inspect":legacy,"record":record,"verify":verify}[a.command](a)
    except Failure as e:out(e)
    except (OSError,ValueError) as e:out(Failure("io_failure",location=type(e).__name__,category="integrity",rc=INTEGRITY))
if __name__=="__main__":main()
