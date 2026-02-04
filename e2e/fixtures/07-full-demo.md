# Full Feature Demo

## Table with Features

| Feature      | Supported |
| ------------ | :-------: |
| `Tables`     |     ✓     |
| **Align**    |     ✓     |
| _Formatting_ |     ✓     |

## Task Lists

- [x] Completed task
- [ ] Pending task
  - Sub list 1
  - Sub list 2

---

## Text Formatting

~~deleted text~~, **bold**, _italic_, `code`, <u>underline</u>

## Math

Inline: $O(n^2)$, and $a^2 + b^2 = c^2$

Display math:

$$
\begin{split}
T_n^2(x) & = T_n(T_n(x)) \\
         & = T_n\left(\frac{a}{x^n}\right) \\
         & = \frac{a}{(\frac{a}{x^n})^n}
\end{split}
$$

## Subscript/Superscript

H<sub>2</sub>O (subscript), E=mc<sup>2</sup> (superscript)

## Blockquote

> A wise quote goes here.
> Multiple lines supported.

## Code

```javascript
const greet = () => console.log("Hello!");
```

```diff
+ added line
- removed line
```

## Links

Auto link: https://github.com

[Inline link](https://google.com) and [reference][ref].

![Local image](./haru.png "avatar")

[ref]: https://google.com
