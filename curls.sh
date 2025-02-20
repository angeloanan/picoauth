curl --unix-socket ./picoauth.sock http://localhost/auth/register -X POST -H "Content-Type: application/json" -d '{"username":"admin","password":"administrator"}' -v
curl --unix-socket ./picoauth.sock http://localhost/auth/login -X POST -H "Content-Type: application/json" -d '{"username":"admin","password":"administrator"}' -v
curl --unix-socket ./picoauth.sock http://localhost/jwt/validate -X POST -d '' -v
curl --unix-socket ./picoauth.sock http://localhost/jwt/refresh -X POST -d '' -v
