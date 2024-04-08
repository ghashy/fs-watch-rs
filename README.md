# fs-watch-rs

This GDExtension is designed for monitoring file system events. It is written using [Rust bindings for Godot 4](https://github.com/godot-rust/gdext) and the [notify](https://github.com/notify-rs/notify) Rust crate.
At a low level, [notify](https://github.com/notify-rs/notify) utilizes FSEvents or kqueue on macOS/iOS, inotify on Linux/Android, and ReadDirectoryChangesW on Windows.

## Using

To use this extension, you need to call `FsWatcher::start` in a separate thread because it blocks the thread to listen to file system events.
To release `FsWatcher`, you need to first call `FsWatcher::stop` and join the thread in which it is running before the `free` call to release the lock from `FsWatcher`.

```gdscript
extends Control

var watcher: FsWatcher
var thread: Thread

func _ready():
    # Store paths on which you want to listen for events
    var arr: Array[String] = ["/Users/ghashy/bin"]
    # Create an instance of `FsWatcher`
    watcher = FsWatcher.from_paths(arr)
    # Connect the signal handler
    watcher.changed.connect(handle_signal)

    # Run `FsWatcher::start` in a new thread
    thread = Thread.new()
    thread.start(run)

func run():
    watcher.start()
    call_deferred("cleanup")

func cleanup():
    thread.wait_to_finish()
    watcher.free()

func handle_signal(event: Dictionary):
    print(event)

func stop():
    watcher.stop()
```
