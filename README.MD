
## 环境变量

> ADDR  默认值 127.0.0.1:8080
>
> MONGODB_URI
> 
> MONGODB_DATABASE
> 

配置 `MONGODB_URI` 和 `MONGODB_DATABASE`后，开启db功能

```bash
export MONGODB_URI='mongodb://192.168.0.3/test?authSource=admin&appName=ws' MONGODB_DATABASE="test"
```

## 查询

**特殊的键**

以`$`开头的键有特殊含义

$gt

$count

$group

$sort

$limit

$skip

**特殊的值**

$exists

$addToSet

$project

