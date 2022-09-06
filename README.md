# w3s-cli 
A simple command line program to upload file or directory to web3.storage with optional encryption and compression

[![downloads](https://img.shields.io/github/downloads/qdwang/w3s-cli/total?style=flat-square)](https://github.com/qdwang/w3s-cli/releases/latest)

## How to use
1. Sign in [https://web3.storage/](https://web3.storage/#).
2. Create an API token and copy it to the clipboard.
3. Download the w3s executable file [here](https://github.com/qdwang/w3s-cli/releases/latest).
4. Open termianl and type:
```shell
w3s remember eyJhbG...lq0(your API token)
```
4. Now you can upload your files. (**Don't upload your sensitive data without encryption**)

## Features
### File upload
```shell
w3s upload-file your_target_file
```

### Directory upload
```shell
w3s upload-dir your_target_dir/
```

### Directory upload with 8 concurrent
```shell
w3s upload-dir -m 8 your_target_dir/
```

### Directory upload with encryption
```shell
w3s -e <password> upload-dir -m 8 your_target_dir/
```

### Directory upload with encryption and compression
```shell
w3s -c -e <password> upload-dir -m 8 your_target_dir/
```
