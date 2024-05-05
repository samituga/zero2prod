Conventions:
.yaml over yml

dependencies:
psql

```shell
sudo apt update & sudo apt upgrade
sudo apt install postgresql postgresql-contrib
```


TODO GitHub workflows
Docker caching doesn't appear to be working
Every job keeps downloading and compiling the rust dependencies, checko if rust cache is working
Consider running everything in a single job
