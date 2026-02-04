# Stress Tests

## Deeply Nested Lists

- Level 1
  - Level 2
    - Level 3
      - Level 4
        - Level 5

## Very Long Line

This is a very long line that should wrap correctly in the editor: Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.

## Adjacent Elements

**bold***italic*`code`~~strike~~

## Empty Elements

-
- non-empty

1.
2. non-empty

## Special Characters in Code

`<script>alert('xss')</script>`

```html
<script>alert('xss')</script>
```

## Consecutive Code Blocks

```javascript
const a = 1;
```
```python
b = 2
```

## Table Without Header Separator

| A | B |
| 1 | 2 |

## Malformed Table (should not render as table)

| A | B
| 1 | 2 |

## Escaped Characters in Links

[link with space](http://example.com/path%20with%20space)

## Multiple Horizontal Rules

---

---

---
