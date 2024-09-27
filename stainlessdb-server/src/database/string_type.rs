use super::Database;
// ストリング型(key, val)型のデータ操作
// SET,GET,DElのコマンドが来た際に&self.string_poolにアクセスする
pub trait StringType {
    type Output;

    fn set(&mut self, key: &str, val: &str) -> Self::Output;
    fn get(&self, key: &str) -> Self::Output;
    fn delete(&mut self, key: &str) -> Self::Output;
}

impl StringType for Database {
    type Output = Option<String>;

    fn set<'a>(&mut self, key: &'a str, val: &'a str) -> Option<String> {
        self.string_pool.insert(key.to_string(), val.to_string())
    }

    fn get(&self, key: &str) -> Option<String> {
        self.string_pool.get(key).map(|val| val.to_owned())
    }

    fn delete(&mut self, key: &str) -> Option<String> {
        self.string_pool.remove(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set() {
        let mut db = Database::new();

        // キーと値のペアをセット
        let old_val = db.set("key1", "value1");
        assert_eq!(old_val, None); // 新しいキーなので以前の値は存在しない

        // 同じキーに新しい値をセット
        let old_val = db.set("key1", "value2");
        assert_eq!(old_val, Some("value1".to_string())); // 前の値が返される
    }

    #[test]
    fn test_get() {
        let mut db = Database::new();

        // キーと値をセット
        db.set("key1", "value1");

        // 値を取得
        let val = db.get("key1");
        assert_eq!(val, Some("value1".to_string()));

        // 存在しないキーを取得
        let val = db.get("key2");
        assert_eq!(val, None); // 存在しないキーはNoneを返す
    }

    #[test]
    fn test_delete() {
        let mut db = Database::new();

        // キーと値をセット
        db.set("key1", "value1");

        // キーを削除
        let deleted_val = db.delete("key1");
        assert_eq!(deleted_val, Some("value1".to_string())); // 削除された値が返される

        // 存在しないキーを削除
        let deleted_val = db.delete("key2");
        assert_eq!(deleted_val, None); // 存在しないキーはNoneを返す
    }
}
