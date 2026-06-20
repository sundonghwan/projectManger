// 블록 content(JSON 문자열) 파싱/생성 헬퍼.
export interface BlockData {
  text: string;
  checked: boolean;
}

export function parseContent(content: string): BlockData {
  try {
    const obj = JSON.parse(content) as Record<string, unknown>;
    return {
      text: typeof obj.text === "string" ? obj.text : "",
      checked: obj.checked === true,
    };
  } catch {
    return { text: "", checked: false };
  }
}

export function stringify(data: BlockData): string {
  return JSON.stringify(data);
}

export function withText(content: string, text: string): string {
  return stringify({ ...parseContent(content), text });
}

export function withChecked(content: string, checked: boolean): string {
  return stringify({ ...parseContent(content), checked });
}
