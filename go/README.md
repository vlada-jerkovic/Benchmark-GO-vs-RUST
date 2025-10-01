# Build & run (Windows PowerShell):

##  Build release
```
go build -ldflags="-s -w" -o splitmix_bench_go.exe main.go
```

## Runs single + multi

### Single-core
```
.\splitmix_bench_go.exe -mode single -total 20000000 -inner 5
```
### Explicit 4 goroutines
```
.\splitmix_bench_go.exe -mode multi -workers 4 -total 20000000 -inner 5
```

### Multi-core (all logical CPUs)
```
.\splitmix_bench_go.exe -mode multi -workers 0 -total 20000000 -inner 5
```