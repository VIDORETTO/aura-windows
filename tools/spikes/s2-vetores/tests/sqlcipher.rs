//! O build com SQLCipher (ADR 0012) mantém o FTS5 com `remove_diacritics 2`
//! e cifra o arquivo: riscos do plano do esforço 047 verificados no spike.
use rusqlite::Connection;

const KEY: &str = "PRAGMA key = \"x'2DD29CA851E7B56E4697B0E1F08507293D761A05CE4D1B628663F411A8086D99'\";";

#[test]
fn fts5_ignora_acentos_com_sqlcipher() {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch(KEY).unwrap();
    c.execute_batch(
        "CREATE VIRTUAL TABLE t USING fts5(texto, tokenize = 'unicode61 remove_diacritics 2');
         INSERT INTO t(texto) VALUES ('fechamos o orçamento de nuvem'), ('reunião de pauta');",
    )
    .unwrap();
    let n: i64 = c
        .query_row("SELECT count(*) FROM t WHERE t MATCH 'orcamento'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);
}

#[test]
fn arquivo_cifrado_nao_abre_sem_a_chave() {
    let dir = std::env::temp_dir().join(format!("s2-cifra-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("m.db");
    {
        let c = Connection::open(&p).unwrap();
        c.execute_batch(KEY).unwrap();
        c.execute_batch("CREATE TABLE x(v); INSERT INTO x VALUES (1);").unwrap();
    }
    let head = std::fs::read(&p).unwrap();
    assert_ne!(&head[..16], b"SQLite format 3\0");
    let sem_chave = Connection::open(&p).unwrap();
    assert!(sem_chave.query_row("SELECT count(*) FROM x", [], |r| r.get::<_, i64>(0)).is_err());
    std::fs::remove_dir_all(&dir).ok();
}
