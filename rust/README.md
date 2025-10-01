# Build & run (Windows PowerShell):

##  Build release
```
cargo build --release
```
## Runs single + multi

###  Single-core
```
.\target\release\splitmix_bench.exe --mode single --total 20000000 --inner 5
```
###  Multi-core (all logical CPUs)
```
.\target\release\splitmix_bench.exe --mode multi --workers 0 --total 20000000 --inner 5
```
###  Explicit 4 threads
```
.\target\release\splitmix_bench.exe --mode multi --workers 4 --total 20000000 --inner 5
```

### Explicit 20 threads
```
.\target\release\splitmix_bench.exe --mode multi --workers 20 --total 20000000 --inner 5
```