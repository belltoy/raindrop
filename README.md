SQL 乱序执行
============

## README

目前实现：假设文件中的 SQL 语言为一行一句。

TODOs:

- [ ] 解析 SQL 语句，支持 SQL 注释


```sh
cargo run -- -h

# Run without excution
env RUST_LOG=debug cargo run -- -i tests/sql

# Run with excution
env RUST_LOG=debug cargo run -- -i tests/sql -e --m 'mysql://root:123456@localhost:3306/db_name'

```

## 题目

TiDB SQL 乱序执行框架

## 描述
在测试 TiDB 事务时，需要多个客户端乱序执行 SQL。

以两个客户端的情况为例。首先启动两个 TiDB 客户端，客户端分别读取对应的 SQL 文件后，以交错顺序执行 SQL 文件中的语句。要求穷举所有执行顺序。

## Example

给定 `sql1.txt`、`sql2.txt`, 内容如下：

```sql
-- sql1.txt:
update X set a=5 where id=1;
update X set a=6 where id=2;
```
```sql
-- sql2.txt
update X set a=8 where id=8
```

启动客户端 client1 读取 `sql1.txt`、client2 读取 `sql2.txt`。

假设 client1 先执行第一条 sql 语句，client2 执行第一条，client1 再执行第二条。则执行顺序是：

```
client1：update X set a=5 where id=1;
client1：update X set a=6 where id=2;
client2：update X set a=8 where id=8;
```

对这个 case，穷举所有可能，意味着执行顺序必须包含以下三种情况：

情况 1：
    client1：update X set a=5 where id=1;
    client1：update X set a=6 where id=2;
    client2：update X set a=8 where id=8;
情况 2：
    client1：update X set a=5 where id=1;
    client2：update X set a=8 where id=8;
    client1：update X set a=6 where id=2;
情况 3：
    client2：update X set a=8 where id=8;
    client1：update X set a=5 where id=1;
    client1：update X set a=6 where id=2;

## 要求

1. 写程序模拟多个客户端实现上述功能
2. 良好的代码设计，可读性，可维护性，可扩展性。
3. 以上可以在单机实现，用 VM 或者 Docker 启动 TiDB 集群不限

