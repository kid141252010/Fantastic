// Minimal DOM for exercising rendered controls and their click handlers in Node.
export class Node {
  constructor(tag = 'div') {
    this.tagName = tag;
    this.children = [];
    this.dataset = {};
    this.attributes = {};
    this.events = {};
    this.className = '';
    this.hidden = false;
    this.style = { setProperty() {}, removeProperty() {} };
    this.classList = {
      toggle: (name, on) => {
        const classes = new Set(this.className.split(' ').filter(Boolean));
        if (on) classes.add(name); else classes.delete(name);
        this.className = [...classes].join(' ');
      },
    };
  }
  set textContent(value) { this.children = [String(value)]; }
  get textContent() { return this.children.map((child) => typeof child === 'string' ? child : child.textContent).join(''); }
  append(...children) { this.children.push(...children); }
  appendChild(child) { this.append(child); return child; }
  replaceChildren(...children) { this.children = children; }
  setAttribute(key, value) { this.attributes[key] = String(value); }
  getAttribute(key) { return this.attributes[key] ?? null; }
  removeAttribute(key) { delete this.attributes[key]; }
  addEventListener(type, fn) { (this.events[type] ||= []).push(fn); }
  click() { if (!this.disabled) return Promise.all((this.events.click || []).map((fn) => fn({ target: this }))); }
  insertAdjacentHTML() {}
  querySelector(selector) { return this.querySelectorAll(selector)[0] || null; }
  querySelectorAll(selector) {
    const matches = (node) => selector.startsWith('#') ? node.id === selector.slice(1)
      : selector.startsWith('.') ? node.className.split(' ').includes(selector.slice(1)) : node.tagName === selector;
    return this.children.flatMap((child) => typeof child === 'string' ? [] : [
      ...(matches(child) ? [child] : []), ...child.querySelectorAll(selector),
    ]);
  }
}
export const document = { createElement: (tag) => new Node(tag), body: new Node('body') };
export function ids(...names) {
  const root = new Node();
  root.append(...names.map((id) => Object.assign(new Node(), { id })));
  return root;
}
