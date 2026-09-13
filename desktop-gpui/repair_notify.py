import io
import re
import subprocess

p = r"crates\webterm\src\app_state.rs"
head = subprocess.run(
    ["git", "show", "HEAD:desktop-gpui/crates/webterm/src/app_state.rs"],
    cwd=r"E:\Coding Stuff\web-term", capture_output=True, text=True, encoding="utf-8"
).stdout.replace("\r\n", "\n")


def find_call_end(src, start):
    """start points at the first char INSIDE Some( — return index of matching close paren."""
    depth = 1
    i = start
    in_str = False
    quote = ""
    while i < len(src):
        c = src[i]
        if in_str:
            if c == "\\":
                i += 2
                continue
            if c == quote:
                in_str = False
        else:
            if c in "\"'":
                in_str = True
                quote = c
            elif c in "([{":
                depth += 1
            elif c in ")]}":
                depth -= 1
                if depth == 0:
                    return i
        i += 1
    return len(src)


# 1. Original statements from HEAD, in order.
orig_stmts = []
for m in re.finditer(r"(this|self)\.notification\s*=\s*Some\(", head):
    start = m.end()  # first char inside Some(
    end = find_call_end(head, start)
    arg = head[start:end]
    orig_stmts.append((m.group(1), arg))
print(f"HEAD originals: {len(orig_stmts)}")

# 2. Current file: locate mangled segments in order.
s = io.open(p, encoding="utf-8").read().replace("\r\n", "\n")
mangled = list(re.finditer(
    r"(this|self)\.notification = Some\((this|self)\.push_notification\(", s))
print(f"mangled sites: {len(mangled)}")
assert len(mangled) == len(orig_stmts), "count mismatch!"

out = []
pos = 0
for m, (recv, arg) in zip(mangled, orig_stmts):
    seg_start = m.start()
    # find end of mangled segment: first ', true, cx);' or ', false, cx);'
    em = re.compile(r", (true|false), cx\)\)?;?").search(s, m.end())
    assert em, "no segment end found"
    seg_end = em.end()
    # consume trailing ')' leftovers from the broken splice
    while seg_end < len(s) and s[seg_end] == ")":
        seg_end += 1
    is_error = "true" if re.search(
        r"Failed|Error|error|Cannot|cannot|Unable|Invalid", arg) else "false"
    repl = f"{recv}.push_notification({arg}, {is_error}, cx);"
    out.append(s[pos:seg_start])
    out.append(repl)
    pos = seg_end
out.append(s[pos:])
s = "".join(out)

# normalize newlines back
io.open(p, "w", encoding="utf-8", newline="").write(s)
print("repaired")
