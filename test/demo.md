# GFM Feature Demos

## Structures

| Feature   | Supported |
| --------- | :-------: |
| `Tables`  |     ✓     |
| **Align** |     ✓     |

- [x] Completed task
- [ ] Pending task
  - Sub list 1
  - Sub list 2

---

## Text

~~deleted text~~, **bold**, _italic_, `code`, <u>underline</u>

Big $O(n^2)$, what about $a^2 + b^2 = c^2$? And $\frac{a^2}{\sqrt{2}}$

$$
\begin{split}
T_n^2(x) & = T_n(T_n(x)) \\
         & = T_n\left(\frac{a}{x^n}\right) \\
         & = \frac{a}{(\frac{a}{x^n})^n} \\
         & = \frac{a (x^n)^n}{a^n} \\
         & = \frac{a x^{n^2}}{a^n} \\
         & = \frac{x^{n^2}}{a^{n-1}} \\
         & = x \frac{x^{n^2 - 1}}{a^{n-1}} \\
         & = x \frac{x^{(n - 1)(n+1)}}{a^{n-1}} \\
         & = x \frac{(x^{n+1})^{n-1}}{a^{n-1}} \\
\end{split}
\tag{2}
$$

H<sub>2</sub>O (subscript), E=mc<sup>2</sup> (superscript)

> block quote

```javascript
// fenced code
const greet = () => console.log("Hello!");
```

```diff
+ added line
- removed line
```

## Links

Auto link: https://github.com

[Inline](https://google.com) and [anchor][gg].

![pic alt](test/haru.png "avatar")

[gg]: https://google.com
