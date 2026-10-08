# Receipt-scope correction

The initial runner hashed every live tracked source path, even unrelated inputs. During the JSON-fixture correction, the format/Python/reproduction receipts captured corrected fixture bytes before its commit while retaining the older Git revision. Own validation rejected that unrelated fixture digest.

Final runtime receipts were rerun after the committed fixture correction and match their pinned inputs. Existing format/Python/reproduction proof is limited to unchanged inputs actually exercised; its unrelated expected-gap-fixture digest is not required or credited. Complete original receipts remain retained; no outcome/log rewrite.

The runner now rejects dirty tracked code inputs and hashes only committed Git blobs from `git archive`. The validator uses an explicit per-command proof scope. No runtime changes or redundant Python/reproduction/format reruns.
