from pathlib import Path
import importlib.util, json, hashlib
R=Path('/home/osso/Projects/wow/wow-ui-sim')
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path); m=importlib.util.module_from_spec(s); s.loader.exec_module(m); return m
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
reports={}
extract=load('extract_current',R/'tools/extract_patch_non_inventory.py')
for v in ['2.1.0','2.0.1']:
 source=R/'data/patch-api/sources'
 evidence=R/'data/patch-api/evidence'/f'{v}-session-2026-10-09'
 raw=(source/f'{v}-api-changes.wikitext').read_text()
 assert raw.encode()==(evidence/'source.wikitext').read_bytes(),v+' exact raw'
 text=(source/f'{v}-api-changes.txt').read_text()
 assert text==extract.extract_text(raw),v+' current extractor default'
 current=json.loads((source/f'{v}-page-coverage.json').read_bytes())
 if v=='2.1.0':
  generator=load('generator_current',R/'tools/gen_patch_wikitext_register.py')
  register=json.loads((source/f'{v}-wikitext-register.json').read_bytes())
  assert register['entries']==generator.parse_legacy_retail_profiling_summary(raw),'current opt-in entries'
  assert register['source']['sha256']==hashlib.sha256(raw.encode()).hexdigest()
  accounting=load('accounting_current',R/'tools/build_patch_2_1_0_accounting.py')
  expected,gaps=accounting.account(register,raw,text)
  assert current==expected,'current 2.1.0 full accounting'
  assert gaps==json.loads((evidence/'historical-known-gaps.json').read_bytes()),'current 2.1.0 exact retained SOURCE gap ledger'
  reports[v]=accounting.counts(register,expected,gaps)
 else:
  auditor=load('auditor_current',R/'tools/audit_patch_2_0_1_source.py')
  expected=auditor.account(raw)
  assert current==expected,'current 2.0.1 literal accounting'
  validator=load('p201_current',evidence/'current/validate.py')
  assert current==json.loads((evidence/'current/page-coverage.json').read_bytes()),'current sealed ledger matches integrated source'
  original=load('p201_original',evidence/'validate.py')
  gaps=original.build_gaps(expected)
  assert gaps==json.loads((evidence/'current/known-gaps.json').read_bytes()),'current 2.0.1 all gap records'
  reports[v]=dict(expected['counts'],gap_records=len(gaps))
checks=[]
for v in ['2.3.0','2.2.0','2.1.0','2.0.1','1.60.1','1.15.9','1.15.8']:
 d=R/'data/patch-api/evidence'/f'{v}-session-2026-10-09'
 for n in ['receipt-seals.json','completion-seals.json','current/seals.json']:
  p=d/n
  if not p.exists():continue
  seals=json.loads(p.read_bytes())
  for name, expected in seals.items():
   dest=R/name if name.startswith('data/') else p.parent/name
   value=expected['sha256'] if isinstance(expected,dict) else expected
   assert sha(dest)==value,str(dest)+' separate receipt seal'
   if isinstance(expected,dict):assert dest.stat().st_size==expected['bytes']
  checks.append({'patch':v,'map':str(p),'members':len(seals),'map_sha256':sha(p)})
 dmap=d/'current-receipts.json'
 if dmap.exists():
  data=json.loads(dmap.read_bytes())
  assert sha(d/'historical-inputs.json')==data['original_manifest_sha256']
  for name,seal in data['separate_current_receipts'].items():
   assert sha(d/name)==seal['sha256'] and (d/name).stat().st_size==seal['bytes']
  checks.append({'patch':v,'map':str(dmap),'members':len(data['separate_current_receipts'])})
print(json.dumps({'current_integrated_accounting':reports,'separate_receipt_maps':checks},indent=2))
