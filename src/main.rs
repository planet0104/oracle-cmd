use oracle::Connection;
use std::env;
use std::io;

fn main() {
    let params = || -> Result<(String, String, String), Box<std::error::Error>> {
        let args:Vec<String> = env::args().collect();
        if args.get(1).is_none(){
            Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
                "请输入数据库名",
            )))
        }else if args.get(2).is_none(){
            Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
                "请输入用户名",
            )))
        }else if args.get(3).is_none(){
            Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
                "请输入链接 例: //192.168.192.15/ocrl",
            )))
        }else{
            Ok((
            args.get(1).unwrap().to_string(),
            args.get(2).unwrap().to_string(),
            args.get(3).unwrap().to_string()))
        }
    }();

    let (db, user, url): (String, String, String) = match params {
        Ok((db, user, url)) => {
            //保存新的配置文件
            let mut conf = ini::Ini::new();
            conf.with_section(Some("Config".to_owned()))
                .set("db", db.clone())
                .set("user", user.clone())
                .set("url", url.clone());
            let _ = conf.write_to_file("conf.ini");
            (db, user, url)
        }
        Err(err) => {
            //读取配置文件
            if let Ok(conf) = ini::Ini::load_from_file("conf.ini") {
                (
                    conf.get_from(Some("Config"), "db")
                        .expect("配置文件读取失败")
                        .to_string(),
                    conf.get_from(Some("Config"), "user")
                        .expect("配置文件读取失败")
                        .to_string(),
                    conf.get_from(Some("Config"), "url")
                        .expect("配置文件读取失败")
                        .to_string(),
                )
            } else {
                println!("{:?}", err);
                return;
            }
        }
    };

    //链接数据库
    println!("connect(\"{}\", \"{}\", \"{}\")...", db, user, url);
    let ret = Connection::connect(&db, &user, &url, &[]);
    if ret.is_err(){
        println!("链接失败: {:?}", ret.err());
        return;
    }
    let conn = ret.unwrap();
    println!("链接成功(输入 quit, exit 退出程序，否则将会导致第二次链接失败！)");
    //查询验证码的sql语句
    loop {
        println!("SQL:");
        let mut cmd = String::new();
        match io::stdin().read_line(&mut cmd) {
            Ok(_n) => {
                let sql = cmd.replace("\r\n", "");
                let sql = sql.trim();
                if sql =="quit" || sql == "exit"{
                    let _ = conn.close();
                    break;
                }else if sql.len() == 0 {
                    continue;
                }
                execute(&conn, &sql);
            }
            Err(error) => println!("{}", error),
        }
    }
    println!("输入Enter键退出.");
    let _ = io::stdin().read_line(&mut String::new());
}

fn execute(conn: &Connection, sql: &str) {
    match conn.prepare(sql,&[]){
        Ok(mut stmt) => {
            match stmt.query(&[]) {
                Ok(rows) => {
                    // for info in rows.column_info() {
                    //     print!("{}|", info.name());
                    // }
                    // println!();
                    let rows:Vec<_> = rows.collect();
                    println!("{}条查询结果", rows.len());
                    for row in &rows {
                        match row{
                            Ok(row) => {
                                for val in row.sql_values(){
                                    let val:String = val.get().unwrap_or("NULL".to_string());
                                    print!("{:15}|", val);
                                }
                                println!();
                            }
                            Err(err) =>{
                                println!("{:?}", err);
                            }
                        }
                    }
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        }
        Err(err) => {
            println!("{:?}", err);
        }
    }
}
