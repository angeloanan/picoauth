# picoauth

> 🤝 | smol opinionated headless authn server

because holy shit other solutions are overkill for my small projects, but also i wanted to know how hard is it to properly handle these

stateless-first -- core feature includes login, logout, and jwt issuance

## features:

- [x] http api
    - [x] http over tcp socket
    - [x] http over unix socket
- [ ] authz
    - [ ] jwt
        - [ ] access tokens
        - [ ] refresh token
    - [ ] cookie
        - [ ] in-memory redis alternative?
        - [ ] refresh cookie
- [x] password login
    - [x] registration
    - [ ] login
    - [ ] password reset
    - [ ] account deletion
    - [ ] hibp checking
- [ ] multi-factor authentication
    - [ ] registration
    - [ ] deletion
    - [ ] time-based one-time passwords
    - [ ] webauthn login
        - [ ] passkey login
        - [ ] FIDO U2F login
- [ ] email verification
- [ ] oauth2 (probably never lol)

## storage method:

- libsql (sqlite)
    - encryption available
- hashed + salted password
    - algorithm secret available

## stateless-first

- ‼️ Session provider
    - JWT is stateless
    - Cookie requires some kind of KV store, could be in-memory or redis or other rust alternative
- ‼️ Storage provider
    - libsql only, other solutions are simply too overkill for my purposes
- ❗ Notification provider
    - email is stateless, requires SMTP config that's set-and-forget
