use log::{error, info};
use mysql::{Opts, Pool, PooledConn, Result as MySQLResult};
use mysql::prelude::Queryable;

pub fn init_clients(url: &str, size: usize) -> MySQLResult<Pool> {
    let opts = Opts::from_url(url)?;
    let pool = Pool::new_manual(size, size, opts)?;
    Ok(pool)
}

pub fn execute_sqls(clients: &mut[(PooledConn, String)], sqls: &[Vec<(&usize, &str)>]) -> MySQLResult<()> {
    let cases_count = sqls.iter().enumerate().map(|(i, case)| {
        info!("Executing case: {}", i);
        let (success, failed): (Vec<_>, _) = case.iter().map(|(idx, sql)| {
            let idx = **idx;
            if let Err(e) = clients[idx].0.query_drop(sql) {
                return Err(format!("Execute sql `{}` error in file: {}, error: {:?}", sql, clients[idx].1, e));
            }
            Ok(())
        })
        .partition(Result::is_ok);

        let failed_count = failed.into_iter().map(Result::unwrap_err).inspect(|e| {
            error!("{}", e);
        }).count();

        if failed_count > 0 {
            error!("Executed errors in case: {}, error count: {:?}, success count: {}", i, failed_count, success.iter().count());
        }
    }).count();

    info!("Executed {} cases", cases_count);

    Ok(())
}
