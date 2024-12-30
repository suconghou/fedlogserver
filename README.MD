
## 环境变量

> ADDR  默认值 127.0.0.1:8080
>
> MONGODB_URI
> 
> MONGODB_DATABASE
> 
> AUTH_KEY , 如果有值则启用ws和aggregate接口鉴权
>

配置 `MONGODB_URI` 和 `MONGODB_DATABASE`后，开启db功能

```bash
export MONGODB_URI='mongodb://192.168.0.3/test?authSource=admin&appName=ws' MONGODB_DATABASE="test"
```

## 查询

**特殊的键**

以`$`开头的键有特殊含义

$gt 开始时间，秒

$lt 结束时间，秒

$count 计数模式，统计符合结果的条数，此键的值作为下标

$group 分组模式，值为`$`开头的字段，统计此字段的分组，不是`$`开头效果等同于`$count`，可以使用包含`%`的时间表示按照时间分组，例如：`%m-%d %H`

$sort 对结果按照字段排序，当值是`$`开头则为升序，否则为降序

$limit 限制返回的大小，值应为数字，区间1-1000

$skip 跳过多少条数据，值应为数字，区间1-1000

**特殊的值**

$exists 代表添加过滤条件，此键必须存在

> 如果值为空（key不是以`$`开头），代表添加过滤条件，此键不存在

$addToSet 分组模式下有效，代表分组时需要附加收集的键值

$project 代表响应数据需要保留此字段，键以`$`开头为保留，否则不保留

其他不以`$`开头的键值对，表示普通过滤条件，等值匹配

