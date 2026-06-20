import { describe, it, expect } from "vitest";
import { parseContent, withText, withChecked, stringify } from "./blockContent";

describe("blockContent", () => {
  it("정상 JSON에서 text/checked 추출", () => {
    expect(parseContent('{"text":"안녕","checked":true}')).toEqual({ text: "안녕", checked: true });
  });

  it("text 없으면 빈 문자열, checked 없으면 false", () => {
    expect(parseContent("{}")).toEqual({ text: "", checked: false });
  });

  it("깨진 JSON은 빈 값으로 안전 처리", () => {
    expect(parseContent("not json")).toEqual({ text: "", checked: false });
  });

  it("withText는 text만 교체하고 checked 유지", () => {
    const next = withText('{"text":"old","checked":true}', "new");
    expect(parseContent(next)).toEqual({ text: "new", checked: true });
  });

  it("withChecked는 checked 토글값 반영", () => {
    const next = withChecked('{"text":"t","checked":false}', true);
    expect(parseContent(next)).toEqual({ text: "t", checked: true });
  });

  it("stringify는 parseContent로 왕복", () => {
    const s = stringify({ text: "x", checked: true });
    expect(parseContent(s)).toEqual({ text: "x", checked: true });
  });
});
