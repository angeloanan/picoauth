curl --unix-socket ./picoauth.sock http://localhost/auth/register -X POST -H "Content-Type: application/json" -d '{"username":"admin","password":"admin"}' -v
curl --unix-socket ./picoauth.sock http://localhost/auth/login -X POST -H "Content-Type: application/json" -d '{"username":"admin","password":"admin"}' -v
