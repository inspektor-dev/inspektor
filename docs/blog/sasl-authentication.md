---
title: password based authentication without TLS using SASL 
description: This blog post explains how SASL works
slug: password-based-authentication-without-tls-using-sasl
authors:
  - name: Poonai
    title: Maintainer of inspektor
    url: https://twitter.com/poonai_
    image_url: https://i.imgur.com/RNM7R6Q.jpg
tags: [postgres, sasl, authentication]
hide_table_of_contents: false
---

## Introduction
TLS is the trusted way of sending messages over a TCP connection. TLS by default encrypts the payload before sending and decrypts after receiving the payload. But if you send plain text on a normal connection, then it can be easily spoofed. 
  
So, if we send a password as plain text in the normal tcp connection, then the attacker can view the password and use the same password to take control of the resource. 

This raises us the question that, how do we authenticate the user on a in-secure connection without revealing the password.

## SASL comes to rescue

This has been solved using [SASL](https://en.wikipedia.org/wiki/Simple_Authentication_and_Security_Layer) (Simple Authentication And Security Layer). If you come from a devops background, you might have noticed SASL error. SASL is used in popular projects like  Postgres, MongoDB, Kafka...

I got to know about SASL, while creating postgres support in inspektor.

In this blog, I'll explain how SCRAM(Salted Challenge Response Authentication Mechanism) works, which is part of SASL family.

## Working of SCRAM
  
#### SCRAM establishes an authenticated connection through a four-step handshake:

**Step 1**: 

Cliend sends nonce (nonce is nothing but randomly chosen bytes) and  user's username to the server to initiate the handshake. This message is called client-first message.
```
client first message. 
n,,n=user,r=fyko+d2lbbFgONRv9qkxdawL
```

**Step 2**:

Server after receiving client-first message, it replies back with its own nonce, salt and iteration count. This message is called the server-first message. 

```
server first message
r=fyko+d2lbbFgONRv9qkxdawL3rfcNHYJY1ZVvWVs7j,s=QSXCR+Q6sek8bf92,
      i=4096
```

**Step 3**: 

Now, client will create `ClientProof` using the parameter from the server-first message to prove that client has right password to authenticate. After creating the `ClientProof`. Client will send the proof to the server. It's called client-final message.

If you are curious about how the proof has been calculated, you can refer the section 3 of SASL RFC (https://datatracker.ietf.org/doc/html/rfc5802#section-3)

```
client final message
c=biws,r=fyko+d2lbbFgONRv9qkxdawL3rfcNHYJY1ZVvWVs7j,
      p=v0X8v3Bz2T0CJGbJQyF0X+HI4Ts=
```

**Step 4**: 

As a final step, server with verify the `ClientProof` that the client has access to the password. After that proof verification completed by the server. 
server will send `ServerSignature` to the client

```
server final message 
v=rmF9pqV8S7suAoZWja4dJRkFsKQ=
```

The `ServerSignature` is used to compare against the `ServerSignature` calculated by the client. This ensures that the client is talking to the correct server.


Now the client has established an authenticated connection without exchanging the password with the server. 

## Conclusion
SASL is not an alternative to TLS, but it can be used along with TLS to harden the authentication process. 
