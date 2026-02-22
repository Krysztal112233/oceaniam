# Security Design

Due to the special nature of this project, various security aspects need to be addressed. These security designs include but are not limited to:

- Storage and management of user credentials
- Storage and management of system user configurations
- Storage and management of application secrets
- Password hashing strength design
- ...and more

## Password Hashing

This design follows the OWASP [Password Storage](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html) recommendations.

Due to the unique nature of password storage—passwords cannot be changed quickly in bulk and are vulnerable to offline attacks once leaked—choosing a reliable password hashing algorithm is essential. In this system design, argon2id is selected as the password hashing algorithm, which provides protection against side-channel attacks and GPU-based attacks.

> [!NOTE]
> Furthermore, slowing down the attacker's cracking speed is crucial.
>
> However, once an attacker has acquired stored password hashes, they are
> always able to brute force hashes offline. Defenders can slow down offline
> attacks by selecting hash algorithms that are as resource intensive as
> possible.

For argon2id, OWASP recommends the following parameters:

- m=12288 (12 MiB), t=3, p=1
- m=9216 (9 MiB), t=4, p=1
- m=7168 (7 MiB), t=5, p=1

These parameters can achieve the same security level, but the trade-off lies in the consumption between CPU and memory.

Considering the request volume, this system chooses to consume more memory to save CPU resources.
