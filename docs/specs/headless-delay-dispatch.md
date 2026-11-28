# Headless delay dispatch

For `dump-tree` and `screenshot`, `--delay` waits for the requested real time after startup and before `--exec-lua`. During that interval, due timers and visible frame `OnUpdate` scripts run with elapsed time. A `Quit()` or `ForceQuit()` request ends the delay early. Without a delay, these commands do not add delay ticks. GUI timing is unchanged.
