# Aeneas 升级调研报告

> 分支：`investigate/aeneas-upgrade`
> 结论：升级可行，但 **translator/ 里的 LLBC 适配器（`llbc/`）需要系统性改动**，aeneas Lean 输出后处理（`lean_post.rs`）需要适配新的 module-system 头。CLI 调用层（`charon_aeneas.rs`）基本不动。

---

## 1. 版本现状与目标

| 组件 | 当前 pin | 目标 pin（最新 nightly） |
|---|---|---|
| aeneas submodule | `f9a8e338` (nightly-2026.09.04) | `557eff83` (nightly-2026.10.02) |
| charon（`aeneas/charon-pin`） | `b82d2748` (nightly-2026.09.04) | `c8f15d7d` (nightly-2026.10.02) |
| commit 差距 | — | aeneas **44** commits，charon **293** commits |

charon 的 commit 号由 aeneas 仓库根目录的 `charon-pin` 文件决定（`aeneas/charon-pin`），aeneas 的 "Update charon" 提交会同步推进它。本地 aeneas 仓库已经带有所需对象（tag 到 `nightly-2026.10.02-557eff8`）；charon 的新 pin 已在本调研中 fetch 到本地。

---

## 2. 影响面总览

translator/ 中有三块对上游版本敏感：

1. **`src/llbc/*.rs`** — 直接读写 charon 产出的 LLBC JSON。**受 charon 格式变化影响最大**。
2. **`src/charon_aeneas.rs`** — 调用 charon / aeneas 二进制。**CLI 参数未变，基本安全**。
3. **`src/lean_post.rs` / `src/lean_inline.rs`** — 对 aeneas 产出的 `.lean` 做后处理（F64→Rat/ℝ、整数模型、内联 helper）。**受 aeneas module-system 迁移影响**。

`simplify.rs`（Rust 源码改写）与 `extract_test.rs`（测试抽取）只碰 Rust 源码，不受版本影响。

---

## 3. 逐项 breaking change

### 3.1 LLBC JSON 格式（charon 侧，最重要）

#### (a) 标量类型：`Literal` → `Scalar`

charon `TyKind::Literal(LiteralTy)` 重构为 `TyKind::Scalar(ScalarTy)`，
`LiteralTy`（`Int`/`UInt`/`Float`/`Bool`/`Char`）重构为
`ScalarTy`（`Integer(IntegerTy)`/`Float`/`Bool`/`Char`），其中
`IntegerTy = Signed(IntTy) | Unsigned(UIntTy)`。

| 旧 JSON | 新 JSON |
|---|---|
| `{"Literal": "Bool"}` | `{"Scalar": "Bool"}` |
| `{"Literal": {"Float": "F64"}}` | `{"Scalar": {"Float": "F64"}}` |
| `{"Literal": {"Int": "I64"}}` | `{"Scalar": {"Integer": {"Signed": "I64"}}}` |
| `{"Literal": {"UInt": "Usize"}}` | `{"Scalar": {"Integer": {"Unsigned": "Usize"}}}` |

**受影响代码**（正则/字符串匹配全部失效，会静默地发现不到目标类型 → adapter 直接 no-op）：
- [f64_adapter.rs:48-59](translation/translator/src/llbc/f64_adapter.rs#L48-L59) `discover_f64_ids`
- [common.rs:27-34](translation/translator/src/llbc/common.rs#L27-L34) `discover_bool_id`
- [i64_adapter.rs:63-77](translation/translator/src/llbc/i64_adapter.rs#L63-L77) `discover_i64_ids`
- [common.rs:417-437](translation/translator/src/llbc/common.rs#L417-L437) `find_bool_dedup`（`"Literal":"Bool"` 字符串匹配）
- [f64_adapter.rs:126-161](translation/translator/src/llbc/f64_adapter.rs#L126-L161) / [i64_adapter.rs:141-176](translation/translator/src/llbc/i64_adapter.rs#L141-L176) `find_bool_in_locals`

#### (b) 常量：`Literal` 枚举并入 `ConstantExprKind`

charon 的常量 `Literal` 枚举（`Scalar`/`Float`/`Bool`/…）被合并进
`ConstantExprKind`（`Bool`/`Integer`/`Float`/`Adt`/`Array`/…），且
`ScalarValue` 改名 `IntegerValue`。浮点常量不再套 `{"Literal": …}` 外壳：

| 旧 JSON（Const 的 kind） | 新 JSON |
|---|---|
| `{"Literal": {"Float": {"value":"1.0","ty":"F64"}}}` | `{"Float": {"value":"1.0","ty":"F64"}}` |
| `{"Literal": {"Scalar": {"Signed":[I64,3]}}}` | `{"Integer": {"Signed":[I64,3]}}` |

（外层 `{"HashConsedValue":[id,[kind,ty]]}` / `{"Deduplicated":id}` 的 hash-cons 机制保持不变。）

**受影响代码**：
- [f64_adapter.rs:343-373](translation/translator/src/llbc/f64_adapter.rs#L343-L373) `build_float_const_id_map`（`k.get("Literal")` 要改成 `k.get("Float")`）
- [f64_adapter.rs:378-396](translation/translator/src/llbc/f64_adapter.rs#L378-L396) `const_float_value`（同上）
- [f64_adapter.rs:447-464](translation/translator/src/llbc/f64_adapter.rs#L447-L464) `extract_float_value`（`lit.get("Float")`/`lit.get("Scalar")` 的取值层级变了）
- [f64_adapter.rs:224-262](translation/translator/src/llbc/f64_adapter.rs#L224-L262) `collect_float_values`

#### (c) `FunId` 被移除

`FnPtrKind::Fun(FunId)`（`FunId = Regular | Builtin`）改为
`FnPtrKind::Fun(FunDeclId)`（`FunDeclId` 直接序列化为裸整数）。

| 旧 JSON（call.func.kind） | 新 JSON |
|---|---|
| `{"Fun": {"Regular": 35}}` | `{"Fun": 35}` |

**受影响代码**：
- [common.rs:149-176](translation/translator/src/llbc/common.rs#L149-L176) `call_stmt` 的 `"kind": {"Fun": {"Regular": func_id}}`

#### (d) `Call` 结构新增 `safety` 字段

`Call` 新增 `safety: CallSafety`（`Inherit`/`Safe`/`Unsafe`）。
缺少该字段会导致 aeneas 反序列化失败。

**受影响代码**：
- [common.rs:149-176](translation/translator/src/llbc/common.rs#L149-L176) `call_stmt` 需要补 `"safety": "Inherit"`

#### (e) `ItemMeta` 新增 3 个字段

`ItemMeta` 新增 `started_from: bool`、`is_extern: bool`、`has_errors: bool`（均无 `serde(default)`，缺失即反序列化失败）。

**受影响代码**：
- [common.rs:73-144](translation/translator/src/llbc/common.rs#L73-L144) `make_fun_decl` 的两条路径（template-clone 与 from-scratch）都要补这 3 个字段（from-scratch 路径最直接，需要硬编码 `false`）。

#### (f) `ordered_decls` 变为 `Option<Vec<…>>`

结构本身仍是 `{"Fun": {"NonRec": id}}`，但现在是 `Option`。adapter 假定它总是数组。

**受影响代码**：
- [f64_adapter.rs:532-551](translation/translator/src/llbc/f64_adapter.rs#L532-L551) / [i64_adapter.rs:228-247](translation/translator/src/llbc/i64_adapter.rs#L228-L247) 插入点查找 + [f64_adapter.rs:619-628](translation/translator/src/llbc/f64_adapter.rs#L619-L628) 写入（需处理 `null`/缺省，例如 `null` → `[]`）。

#### (g) 未变化的（已核实，无需改）

- `fun_decls` 仍是按 def_id 索引的**数组**（`IndexMap` 序列化为连续 Vec）。
- `item_names` / `short_names` 仍是 `[{"key":…,"value":…}]` 数组（`SeqHashMapToArray`）。
- span 的 JSON 形状不变（`SerializedSpan{data{file_id,beg{line,col},end{line,col}}, generated_from_span}`），"tiny spans" 只是内存表示。
- `GenericParams`（7 个字段）、`GenericArgs`（4 个字段）、`FunSig`（5 个字段）字段集不变。
- `FunDecl.src: "Normal"` 仍有效（`FunSource::Normal`）。

### 3.2 aeneas CLI（基本安全）

`-sequential`、`-all-computable`、`-backend lean`、`-loops-to-rec` 在新 pin 均存在且语义一致（见 `src/Main.ml`）。`charon_aeneas.rs` 的 aeneas 调用无需改动。

### 3.3 aeneas Lean 输出（module-system 迁移，PR #1230）

生成文件头从：

```lean
import Aeneas
set_option linter.dupNamespace false
...
```

变为：

```lean
module
public import Aeneas
@[expose] public section
open Aeneas Aeneas.Std Result ControlFlow Error
set_option linter.dupNamespace false
...
```

**受影响代码**：
- [lean_post.rs:216-217](translation/translator/src/lean_post.rs#L216-L217) `lean_f64_to_real` 的 `import Aeneas\n` 正则**静默失配** → Real 模式不会插入 `import Mathlib` → 编译失败。需要改成匹配 `public import Aeneas`（或更稳的锚点）。
- 浮点/整数命名**不变**：`float_name` 在旧/新 pin 都产出 `F64`；`Std.I8/U32/…`、`core.num.U*.wrapping_*`、`core.clone.Clone*`、`namespace <crate>` 均保留。`F64→Rat/ℝ` 替换链（`std.f64.F64.sqrt` → `std.f64.Rat.sqrt`）不受影响。
- `@[expose]` 属性在生成文件中出现（PR #1362），后处理需注意不要破坏它。

### 3.4 本地未提交的 patch 会失效

aeneas submodule 里有一个未提交改动：`backends/lean/Aeneas/Tactic/Step/Step.lean` 的
`extractCallSiteTree` 增加一行 `Std.WP.qimp_dspec` 匹配。目标版本中 `qimp_spec`/`qimp_dspec`/`qimp`
整族已被 PR #1383 删除，且 `extractCallSiteTree` 本身已被重写（不再按 `qimp_*` 形状 match），
该 patch 应直接丢弃（如仍需，需按新版 `Step.lean` 结构重新定位）。详见
[translation/local/AENEAS_PATCH.md](translation/local/AENEAS_PATCH.md)。

---

## 4. 需要改动的文件清单（估算）

| 文件 | 改动类型 |
|---|---|
| `translation/translator/src/llbc/common.rs` | 3 处正则/字符串（bool 类型发现）+ `call_stmt`（Fun + safety）+ `make_fun_decl`（3 个 ItemMeta 字段）+ `find_bool_dedup` |
| `translation/translator/src/llbc/f64_adapter.rs` | `discover_f64_ids` 正则 + 常量发现/替换（Literal→ConstantExprKind）+ `ordered_decls` Option |
| `translation/translator/src/llbc/i64_adapter.rs` | `discover_i64_ids` 正则（Int→Integer/Signed）+ `find_bool_in_locals` + `ordered_decls` Option |
| `translation/translator/src/lean_post.rs` | `import Aeneas` → `public import Aeneas`（Mathlib 插入点） |
| `translation/translator/src/charon_aeneas.rs` | 预计无改动 |
| aeneas submodule 本地 patch | 丢弃 `Step.lean` 的 `qimp_dspec` 改动（见 [local/AENEAS_PATCH.md](translation/local/AENEAS_PATCH.md)） |

> 注：`llbc/` 的适配器是"字符串/JSON 结构假设"式实现，没有用 charon 的 Rust AST 定义来解析。因此以上每一处都是**静默失效**（发现不到类型 → 直接 no-op，或注入的声明缺字段 → aeneas 报反序列化错）。修复后必须用实际 LLBC 样本验证注入的 helper 能被 aeneas 消费。

---

## 5. 升级步骤（供后续执行）

```bash
# 1. 更新 aeneas submodule 到新 pin
cd /home/disk2/liguchan/Rust/autospec/aeneas
git checkout 557eff83ecef5083b98a52a94ca7fae63d6c1dab

# 2. 丢弃本地 Step.lean patch（见 translation/local/AENEAS_PATCH.md）
git checkout -- backends/lean/Aeneas/Tactic/Step/Step.lean

# 3. 升级 charon + 构建 aeneas（无需手动碰 charon checkout）
make setup-charon   # = ./scripts/check-charon-install.sh --force
                    #   读 charon-pin → git fetch && git checkout <pin> → rebuild charon (make test)
make                # = build → build-dev → build-bin/build-lib/build-bin-dir
                    #   每个 target 依赖 check-charon（校验 pin 与 bin/charon）

# 4. 更新外层仓库的 submodule 指针并提交
cd /home/disk2/liguchan/Rust/autospec
git add aeneas

# 5. 修复 translator（见第 4 节），重建并回归
cargo build --manifest-path translation/translator/Cargo.toml
# 用 translation/local/test_rust 或 lean_mode 下的用例回归
```

要点：`make setup-charon` 会**自动**按新的 `charon-pin` 把 `./charon` 检出到对应
commit 并重建 charon；`make` 会构建 `bin/aeneas`。因此**不需要手动在 `aeneas/charon`
里 checkout**。charon 重建仍需要对应 `rust-toolchain` 与 OCaml 环境。

---

## 6. 风险与建议

- **最大风险在 `llbc/` 适配器**：它不是用 charon 的 serde 类型反序列化，而是手写 JSON 假设。charon 一个月 293 个 commit 里包含多次 AST 重构（`Literal`→`Scalar`、`Literal` 并入 `ConstantExprKind`、`FunId` 移除等），几乎每一处手写假设都要同步。
- **建议优先做一次"实跑验证"**：先只更新 charon+aeneas 二进制，用现有 `translation/local/test_rust`（含 f64/i64）跑一遍，抓取新版 `.llbc` 与 `.lean` 的实际字节，再据此精确改写适配器 —— 本报告里的 JSON 形状是从 serde 定义反推的，`HashConsed`/`serde_state` 细节应以实际输出为准。
- **`lean_post.rs` 的 header 适配是小改动**，但 Real 模式的 `import Mathlib` 失配是静默的，回归时务必覆盖 `float-model=Real`。
- 本次调研**未执行构建与实跑**（构建 charon+aeneas 成本高），上述结论为源码级核实，动手前建议先做第 5 节的步骤 1–2 并抓样例验证。
