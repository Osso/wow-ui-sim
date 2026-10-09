from pathlib import Path
import hashlib, json, os, shutil, time, tarfile
R = Path('/home/osso/Projects/wow/wow-ui-sim')
O = Path('/tmp/era1134-1133-independent')
REV = '9252c6cc93c087518f25e64987e095ed76312a51'
def digest(data): return hashlib.sha256(data).hexdigest()
def save(name, obj): (O/name).write_text(json.dumps(obj, indent=2)+'\n')
def snapshot():
    paths = []
    for v in ['1.13.4','1.13.3']:
        paths += list((R/'data/patch-api/evidence'/f'{v}-session-2026-10-09').rglob('*'))
    paths += list((R/'src').rglob('*.rs')) + list((R/'src').rglob('*.lua'))
    paths += [R/'Cargo.toml', R/'Cargo.lock', R/'patch-tests/patch_1_13_4_totems.rs', R/'patch-tests/patch_1_13_3_npc_health.rs']
    return {str(p.relative_to(R)):digest(p.read_bytes()) for p in sorted(set(paths)) if p.is_file()}
def command(label, argv, overrides=None):
    overrides = overrides or {}
    env = dict(os.environ, **overrides)
    save(label+'.env.json', env)
    start = time.time()
    result = cli.command(*argv).cwd(str(R)).env(overrides).capture().run()
    end = time.time()
    (O/(label+'.stdout')).write_text(result.stdout)
    (O/(label+'.stderr')).write_text(result.stderr)
    receipt = dict(label=label,argv=argv,cwd=str(R),revision=REV,start_epoch=start,end_epoch=end,exit_code=result.exit_code,environment_file=label+'.env.json',environment_sha256=digest((O/(label+'.env.json')).read_bytes()),overrides=overrides,stdout=label+'.stdout',stderr=label+'.stderr')
    save(label+'.receipt.json',receipt)
    with (O/'report.md').open('a') as report:
        report.write('\n## '+label+'\n```json\n'+json.dumps(receipt,indent=2)+'\n```\n### Full stdout\n```text\n'+result.stdout+'\n```\n### Full stderr\n```text\n'+result.stderr+'\n```\n')
    print(label, 'exit',result.exit_code,'seconds',round(end-start,3),'full streams saved')
    return result
(O/'report.md').write_text('# Independent integrated Era 1.13.4 / 1.13.3 proof\n\nScope: owned SOURCE, default replay, scratch-only serialized tamper controls, original/receipt seals, separate actual 1.13.4 successor inspection for frozen 1.13.3, and exactly one two-target offline/locked headless Cargo invocation. No runtime fix, historical signature/native/loaded-UI claim; no other Era targets, broad check, full suite, network, commit, deployment or delegation.\n')
save('before-hashes.json',snapshot())
command('revision',['git','rev-parse','HEAD'])
command('before-status',['git','status','--short'])
command('integration-parents',['git','show','--no-patch','--format=%H %P %s','03fa877e6','0c5cecbf9'])
command('root-delta',['git','show','9252c6cc9','--','src/lua_api/workarounds/temporary/housing_catalog_state.rs'])
seal_results={}
for v in ['1.13.4','1.13.3']:
    d=R/'data/patch-api/evidence'/f'{v}-session-2026-10-09'
    checks={}
    for name in ['seals.json','receipt-seals.json']:
        seal=json.loads((d/name).read_bytes())
        failures=[key for key,value in seal.items() if not (d/key).is_file() or digest((d/key).read_bytes()) != value]
        checks[name]=dict(count=len(seal),failures=failures,map_sha256=digest((d/name).read_bytes()))
    with tarfile.open(d/'originals.tar.gz') as archive:
        members=archive.getmembers()
        archived_map=json.load(archive.extractfile('seals.json'))
        failures=[key for key,value in archived_map.items() if digest(archive.extractfile(key).read()) != value]
        checks['archive']=dict(members=len(members),sealed_count=len(archived_map),failures=failures,map_matches_original=archived_map==json.loads((d/'seals.json').read_bytes()))
    seal_results[v]=checks
    copied=O/('evidence-'+v)
    shutil.copytree(d,copied)
    command('source-'+v,['python3','-B',str(copied/'test_source_accounting.py')],{'PYTHONDONTWRITEBYTECODE':'1'})
    if v=='1.13.4':
        command('successors-'+v,['python3','-B',str(copied/'test_successors.py')],{'PYTHONDONTWRITEBYTECODE':'1'})
    argv=['python3','-B',str(copied/'test_portable.py'),'--archive',str(copied/'originals.tar.gz'),'--scratch',str(O/('scratch-'+v)),'--receipts',str(O/('portable-'+v+'.json'))]
    if v=='1.13.3': argv += ['--cwd',str(R)]
    command('portable-'+v,argv,{'PYTHONDONTWRITEBYTECODE':'1'})
    command('validator-'+v,['python3','-B',str(copied/('validator.py' if v=='1.13.4' else 'audit.py'))],{'PYTHONDONTWRITEBYTECODE':'1'})
save('seal-results.json',seal_results)
with (O/'report.md').open('a') as f: f.write('\n## Independently hashed original/receipt/archive seals\n```json\n'+json.dumps(seal_results,indent=2)+'\n```\n')
command('combined-era',['cargo','test','--offline','--locked','--no-default-features','--features','client-era','--test','patch_1_13_4_totems','--test','patch_1_13_3_npc_health','--','--nocapture'],{'CARGO_NET_OFFLINE':'true','WOW_SIM_NO_ADDONS':'1','WOW_SIM_NO_SAVED_VARS':'1','WOW_SIM_P1133_HEALTH_OUT':str(O/'npc-health-observations.json')})
save('after-hashes.json',snapshot())
command('after-revision',['git','rev-parse','HEAD'])
command('after-status',['git','status','--short'])
before=json.loads((O/'before-hashes.json').read_text()); after=json.loads((O/'after-hashes.json').read_text())
save('hash-delta.json',{'added':sorted(after.keys()-before.keys()),'removed':sorted(before.keys()-after.keys()),'changed':[k for k in before.keys()&after.keys() if before[k]!=after[k]],'files_before':len(before),'files_after':len(after)})
print('hash delta', (O/'hash-delta.json').read_text())
