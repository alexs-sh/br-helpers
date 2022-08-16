### About

Buidlroot helper(s). Some additional tools to automate and simplify work process
with Buildroot. WIP.


[![Build Status](https://gitlab.com/alexssh/br-helpers/badges/master/pipeline.svg)](https://gitlab.com/alexssh/br-helpers/-/commits/master)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)


### brdiff

The app collects information about the packages and generates reports. The
reports could be in a short form, for example, only information about changing
versions of packages. Or in full form. The full reports are only available for
git-based packages and contain information about commit id, summary & author.

The brdiff uses JSON or mk files as input. The JSON files could be generated by
Buildroot's **show-info** command. 

The mk files are usually located in **package** directory in a Buildroot or in
a Buildroot's external tree. 

#### Examples
Here are several examples of how to generate reports. In these examples, we have 2 different directories with Buildroot that were built in a different
time,  with different configs, different Buildroot versions, etc.


**Preparing JSON files**

```
cd /buildroot-orig
make show-info > report.json

cd /buildroot-mod
make show-info > report.json
```

**Compare JSON reports. Short**

Run fast comparison without getting information from VCS.

```
brdiff -f /buildroot-orig/report.json -s /buildroot-mod/report.json
[*] uuu [modified]
      version: 30e5d5722035dd75e6d4749040a212524bb2f629 -> 294ab5c377ae00c0e659c03bb7cc6eef40e724df
[*] libopenssl [modified]
      version: a644cb7c1c19c78e2ca393c8ca36989e7ca61715 -> 9574842e90e29015daa2b071e965cec9aa885c17
```

**Compare JSON reports. Full**

Run detailed comparison based on information from VCS.

```
brdiff brdiff -f /buildroot-orig/report.json -s /buildroot-mod/report.json -m full
[*] uuu [modified]
      version: 30e5d5722035dd75e6d4749040a212524bb2f629 -> 294ab5c377ae00c0e659c03bb7cc6eef40e724df
       - fix fail open file begin with > in script
           - id: 294ab5c377ae00c0e659c03bb7cc6eef40e724df
           - author: Frank Li <frank.li@nxp.com>
       - fix build failure at windows
           - id: 4bf291e91330cfabadd4e7f7be28c2dfaacbea6b
           - author: Frank Li <frank.li@nxp.com>
       - fix warning at trans.cpp and usbhotplug.cpp
           - id: e44daeb3bcc4c3663a43f95cf1e361c2370376e6
           - author: Frank Li <Frank.Li@nxp.com>
       - fix warning at sdp.cpp
           - id: ce351f03c5dcd8baa4a3ecfc1dfb74129f4edd1a
           - author: Frank Li <Frank.Li@nxp.com>
       - fix warning at cmd.cpp
         ...
[*] libopenssl [modified]
      version: a644cb7c1c19c78e2ca393c8ca36989e7ca61715 -> 9574842e90e29015daa2b071e965cec9aa885c17
      sources: changed
       - Pre-declare all core dispatch table functions, and fix the internal ones
           - id: 9574842e90e29015daa2b071e965cec9aa885c17
           - author: Richard Levitte <levitte@openssl.org>
       - add a check for the return of OBJ_new_nid()
           - id: a0ff8e413e94ba46720a4bf3a5032c50531c526c
           - author: xkernel <xkernel.wang@foxmail.com>
       - ci: add GitHub token permissions for workflows
           - id: c6e7f427c82dfa17416a39af7661c40162d57aaf
           - author: Varun Sharma <varunsh@stepsecurity.io>
       - OSSL_trace_set_channel.pod and openssl.pod: fix missing/inconsistent category items
           - id: 6d594fdf52c4824acff9a1e50e2e2ea576a64fd1
           - author: Dr. David von Oheimb <David.von.Oheimb@siemens.com>
       - x509_vfy.c: Revert the core of #14094 regarding chain_build() error reporting
           - id: 1f00dc4f8c0ef0101368de2adf22495e5e295114
           - author: Dr. David von Oheimb <David.von.Oheimb@siemens.com>
           ...
```


**Compare MK files**

There are two variants: 

- compare two files
- compare two directories with mk files

If paths to files are specified as input parameters, then brdiff will compare two files. If directories, then brdiff will search all mk files in directories and compare them.

```
brdiff -f /orig/libopenssl.mk -s /mod/libopenssl.mk -m full
[*] libopenssl [modified]
      version: OpenSSL_1_1_1a -> OpenSSL_1_1_1j
       - Prepare for 1.1.1j release
           - id: 52c587d60be67c337364b830dd3fdc15404a2f04
           - author: Matt Caswell <matt@openssl.org>
       - Update copyright year
           - id: 2b2e3106fc57b810d91221aef4c4c39a8afd97c3
           - author: Matt Caswell <matt@openssl.org>
       - Update CHANGES and NEWS for new release
           - id: 8b02603cedc8fbdf9901aa2cc71877c28adbcaf2
           ...
```

```
brdiff -f /orig/ -s /mod/ -m full
[*] libopenssl [modified]
      version: OpenSSL_1_1_1a -> OpenSSL_1_1_1j
       - Prepare for 1.1.1j release
           - id: 52c587d60be67c337364b830dd3fdc15404a2f04
           - author: Matt Caswell <matt@openssl.org>
       - Update copyright year
           - id: 2b2e3106fc57b810d91221aef4c4c39a8afd97c3
           - author: Matt Caswell <matt@openssl.org>
       - Update CHANGES and NEWS for new release
           - id: 8b02603cedc8fbdf9901aa2cc71877c28adbcaf2
           ...
[*] uuu [modified]
      version: uuu_1.4.191 -> uuu_1.4.240
       - Fix ZSTD stopping
           - id: 306415147280ec9b9d6859d3174566ad8e509b36
           - author: chriswu86459 <christopher.wu@nxp.com>
       - fix fail open file begin with > in script
           - id: 294ab5c377ae00c0e659c03bb7cc6eef40e724df
           - author: Frank Li <frank.li@nxp.com>
       - fix build failure at windows
           - id: 4bf291e91330cfabadd4e7f7be28c2dfaacbea6b
           - author: Frank Li <frank.li@nxp.com>
       - fix warning at trans.cpp and usbhotplug.cpp
           - id: e44daeb3bcc4c3663a43f95cf1e361c2370376e6
           - author: Frank Li <Frank.Li@nxp.com>
       - fix warning at sdp.cpp
           - id: ce351f03c5dcd8baa4a3ecfc1dfb74129f4edd1a
           - author: Frank Li <Frank.Li@nxp.com>
           ... 

```
