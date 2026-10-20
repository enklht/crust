use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TEST: AtomicUsize = AtomicUsize::new(0);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "crust-updates-{}-{}",
            std::process::id(),
            NEXT_TEST.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_runs(source: &str) {
    let dir = TestDir::new();
    let input = dir.0.join("test.c");
    let executable = dir.0.join("test");
    fs::write(&input, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_crust"))
        .arg(&input)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let output = Command::new("gcc")
        .arg(input.with_extension("s"))
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let output = Command::new(&executable).output().unwrap();
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn compound_shifts_preserve_left_type() {
    assert_runs(
        r#"
        int main(void) {
            long wide = 4294967296L;
            if ((wide >>= 1) != 2147483648L) return 1;
            int negative = -4;
            if ((negative >>= 1U) != -2) return 2;
            wide = 1L;
            if ((wide <<= 40) != 1099511627776L) return 3;
            int narrow = 3;
            if ((narrow <<= 2L) != 12) return 4;
            return 0;
        }
        "#,
    );
}

#[test]
fn updates_evaluate_lvalues_once_and_preserve_results() {
    assert_runs(
        r#"
        int value;
        int calls;
        int *get(void) { calls += 1; return &value; }
        int main(void) {
            value = 3;
            if ((*get() += 4) != 7 || calls != 1 || value != 7) return 1;
            if ((*get())++ != 7 || calls != 2 || value != 8) return 2;
            if (--*get() != 7 || calls != 3 || value != 7) return 3;
            if ((*get())-- != 7 || calls != 4 || value != 6) return 4;
            if (++*get() != 7 || calls != 5 || value != 7) return 5;
            int other = 2;
            if ((*get() += (other *= 3)) != 13 || other != 6 || calls != 6) return 6;
            if (0 && (*get() += 100)) return 7;
            if ((1 ? other++ : (*get())++) != 6) return 8;
            if (calls != 6 || value != 13 || other != 7) return 9;
            double rounded = 9007199254740992.0;
            if (rounded++ != 9007199254740992.0) return 10;
            unsigned int wrap = 4294967295U;
            if (wrap++ != 4294967295U || wrap != 0U) return 11;
            int mixed = 3;
            if ((mixed *= 2.5) != 7 || mixed != 7) return 12;
            return 0;
        }
        "#,
    );
}

#[test]
fn sequence_bindings_handle_nested_updates_and_shadowing() {
    assert_runs(
        r#"
        int main(void) {
            int x = 1;
            int y = 10;
            int which = 0;
            if ((*(++which ? &x : &y) += (y += 2)) != 13) return 1;
            if (x != 13 || y != 12 || which != 1) return 2;
            {
                double x = 2.5;
                if (x++ != 2.5 || x != 3.5) return 3;
            }
            if (x++ != 13 || x != 14) return 4;
            return 0;
        }
        "#,
    );
}

#[test]
fn updates_still_require_lvalues_and_valid_operands() {
    let dir = TestDir::new();
    let input = dir.0.join("invalid.c");
    for expression in ["(1 + 2)++", "++(++x)", "(x += 1) += 2", "x <<= 1.5"] {
        fs::write(
            &input,
            format!("int main(void) {{ int x = 0; {expression}; return x; }}"),
        )
        .unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_crust"))
            .arg("--validate")
            .arg(&input)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{expression}: {output:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("Validation error:"));
    }
}
