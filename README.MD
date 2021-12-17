
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

**match 参数**

> type
>
> ip
>
> cookie
>
> ua
>
> href
>
> target
>
> message
>
> refer

> hours 限定最近n小时

**计数模式**

计算match后的条数

> count:'num'

**group模式**

> group:"ip"
>
> sets

**分页**

> skip
>
> limit


