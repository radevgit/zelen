# Migrating FlatZinc-Exported Files to New Selen API

## Overview

If you have auto-generated Selen programs from FlatZinc (like `agprice_full.rs`), they use the **old type-specific API** that has been removed. This guide shows how to update these files to work with the **new generic API**.

## Problem Identification

Old FlatZinc exports use methods like:
```rust
model.float_lin_eq(&coeffs, &vars, rhs);
model.float_lin_le(&coeffs, &vars, rhs);
model.int_lin_eq(&coeffs, &vars, rhs);
model.int_lin_le(&coeffs, &vars, rhs);
```

These methods **no longer exist** in Selen. They have been replaced with generic methods that work for both int and float.

## Quick Fix Guide

### 1. Replace Old Linear Constraint Methods

**Find and Replace:**

| Old Method | New Method |
|------------|-----------|
| `model.float_lin_eq(` | `model.lin_eq(` |
| `model.float_lin_le(` | `model.lin_le(` |
| `model.float_lin_ne(` | `model.lin_ne(` |
| `model.int_lin_eq(` | `model.lin_eq(` |
| `model.int_lin_le(` | `model.lin_le(` |
| `model.int_lin_ne(` | `model.lin_ne(` |

**Example:**
```rust
// OLD (won't compile):
model.float_lin_eq(&vec![420.0, 1185.0, 6748.0, -1.0], 
                   &vec![cha, butt, milk, revenue], 
                   0.0);

// NEW (works):
model.lin_eq(&vec![420.0, 1185.0, 6748.0, -1.0], 
             &vec![cha, butt, milk, revenue], 
             0.0);
```

### 2. Replace Reified Constraint Methods

If your FlatZinc export uses reified constraints:

| Old Method | New Method |
|------------|-----------|
| `model.int_eq_reif(` | `model.eq_reif(` |
| `model.float_eq_reif(` | `model.eq_reif(` |
| `model.int_ne_reif(` | `model.ne_reif(` |
| `model.float_ne_reif(` | `model.ne_reif(` |
| `model.int_le_reif(` | `model.le_reif(` |
| `model.float_le_reif(` | `model.le_reif(` |
| `model.int_lt_reif(` | `model.lt_reif(` |
| `model.float_lt_reif(` | `model.lt_reif(` |

**Example:**
```rust
// OLD:
model.int_eq_reif(x, 5, bool_var);

// NEW:
model.eq_reif(x, 5, bool_var);
```
