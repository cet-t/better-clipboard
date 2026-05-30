export class Combo {
  private wrap: HTMLElement;
  private input: HTMLInputElement;
  private list: HTMLElement;
  private options: string[] = [];
  private filtered: string[] = [];
  private selected = "";
  private activeIdx = -1;
  onChange?: (value: string) => void;

  constructor(container: HTMLElement, options: { placeholder?: string } = {}) {
    this.wrap = container;
    this.wrap.classList.add("combo-wrap");

    this.input = document.createElement("input");
    this.input.className = "combo-input";
    this.input.type = "text";
    this.input.placeholder = options.placeholder || "";
    this.wrap.appendChild(this.input);

    this.list = document.createElement("div");
    this.list.className = "combo-list";
    this.wrap.appendChild(this.list);

    this.input.addEventListener("input", () => this.onInput());
    this.input.addEventListener("focus", () => this.open());
    this.input.addEventListener("blur", () => this.close());
    this.input.addEventListener("keydown", (e) => this.onKey(e));
  }

  setOptions(opts: string[]) {
    this.options = opts;
    this.filtered = opts;
    this.render();
  }

  setValue(val: string) {
    this.selected = val;
    this.input.value = val;
  }

  getValue(): string {
    return this.selected;
  }

  private onInput() {
    const q = this.input.value.toLowerCase();
    this.filtered = this.options.filter((o) => o.toLowerCase().includes(q));
    this.activeIdx = -1;
    this.open();
    this.render();
  }

  private onKey(e: KeyboardEvent) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      this.activeIdx = Math.min(this.activeIdx + 1, this.filtered.length - 1);
      this.render();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      this.activeIdx = Math.max(this.activeIdx - 1, 0);
      this.render();
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (this.activeIdx >= 0 && this.activeIdx < this.filtered.length) {
        this.select(this.filtered[this.activeIdx]);
      }
    } else if (e.key === "Escape") {
      this.input.blur();
    }
  }

  private select(val: string) {
    this.selected = val;
    this.input.value = val;
    this.close();
    this.onChange?.(val);
  }

  private open() {
    const q = this.input.value.toLowerCase();
    this.filtered = this.options.filter((o) => o.toLowerCase().includes(q));
    this.render();
    this.list.classList.add("open");
  }

  private close() {
    setTimeout(() => this.list.classList.remove("open"), 100);
  }

  private render() {
    this.list.innerHTML = "";
    this.filtered.forEach((opt, i) => {
      const div = document.createElement("div");
      div.className = "combo-option";
      if (opt === this.selected) div.classList.add("selected");
      if (i === this.activeIdx) div.classList.add("active");
      div.textContent = opt;
      div.addEventListener("mousedown", (e) => {
        e.preventDefault();
        this.select(opt);
      });
      this.list.appendChild(div);
    });
  }
}
