# w3s-cli 
[![release](https://img.shields.io/github/v/release/qdwang/w3s-cli?include_prereleases&style=flat-square)](https://github.com/qdwang/w3s-cli/releases/latest)

A simple command line program to upload file or directory to web3.storage with optional encryption and compression. 

<img width="650" alt="w3s-cli" src="https://user-images.githubusercontent.com/403616/189143211-6255cf9d-e483-4fe2-9175-11270c31ffd7.png">

## Features

* Uploads single file to web3.storage
* Uploads entire directory recursively to web3.storage
* Uploads with optional encryption and compression
* Encryption and compression during uploading without pre process needed
---
* Downloads single file from IPFS gateway
* Downloads entire directory recursively from IPFS gateway
* Downloads with optional decryption and decompression
* Encryption and compression during downloading without after process needed


## Preparation
1. Sign in [https://web3.storage/](https://web3.storage/#).
2. Create an API token and copy it to the clipboard.
3. Download the w3s executable file [here](https://github.com/qdwang/w3s-cli/releases/latest).
4. Open termianl and type:
```shell
w3s remember eyJhbG...lq0(your API token)
```
5. Now you can upload your files. (**Don't upload your sensitive data without encryption**)



