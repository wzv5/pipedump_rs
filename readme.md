# pipedump

由于需要调试管道通信，又找不到方便好用的工具，就随手写了一个。

（虽然两个 `tee` 确实可以用，但感觉太不优雅）

``` bash
pipedump_rs ping 127.0.0.1
```

命令行参数会按原样传递给目标程序，管道通信的内容会保存到 `$temp/pipedump/` 目录下。

只依赖 `libc`，应该可以兼容所有系统。

## License

WTFPL
