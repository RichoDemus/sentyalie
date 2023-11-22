# Sentyalië

## Running
Set envs `DISCORD_TOKEN` and `DISCORD_CHANNEL` start and curl /run (or /get to just test)
## invite bot to server
https://discord.com/api/oauth2/authorize?client_id=888455645681045604&scope=bot&permissions=3072

## manually trigger bot once:
```
curl -H "Authorization: Bearer $(gcloud auth print-identity-token)" https://sentyalie-c7j3dfx7ra-uc.a.run.app/get
```

## deploying
1. just push, GH action will create and upload container
2. go to https://console.cloud.google.com/run and deploy a new revision
