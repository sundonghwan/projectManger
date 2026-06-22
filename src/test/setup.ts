// Vitest 전역 셋업 — Testing Library의 jest-dom 매처 등록
import "@testing-library/jest-dom/vitest";

// Node 22+/26 의 내장 localStorage 글로벌이 (--localstorage-file 미지정으로) undefined 라서
// jsdom 의 localStorage 까지 가려진다. 테스트 한정 인메모리 폴리필로 보정한다.
if (typeof globalThis.localStorage === "undefined" || globalThis.localStorage === null) {
  const store = new Map<string, string>();
  const mock: Storage = {
    getItem: (k) => (store.has(k) ? store.get(k)! : null),
    setItem: (k, v) => {
      store.set(k, String(v));
    },
    removeItem: (k) => {
      store.delete(k);
    },
    clear: () => {
      store.clear();
    },
    key: (i) => Array.from(store.keys())[i] ?? null,
    get length() {
      return store.size;
    },
  };
  Object.defineProperty(globalThis, "localStorage", {
    value: mock,
    configurable: true,
    writable: true,
  });
}
