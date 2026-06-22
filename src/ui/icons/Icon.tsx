import type { CSSProperties, SVGProps } from "react";
import { ICON_PATHS, type IconName } from "./paths";

export type { IconName };

export interface IconProps extends Omit<SVGProps<SVGSVGElement>, "name"> {
  name: IconName;
  /** px 단위 정사각 크기 (기본 16) */
  size?: number;
  strokeWidth?: number;
}

/**
 * 단색 아웃라인 아이콘. stroke=currentColor 로 부모 텍스트 색을 상속해
 * 라이트/다크 테마에 자동 대응한다. 접근성 라벨은 보통 부모 버튼에 있으므로
 * 기본적으로 aria-hidden 처리한다(aria-label 을 주면 노출).
 */
export function Icon({ name, size = 16, strokeWidth = 1.75, style, ...rest }: IconProps) {
  const merged: CSSProperties = { display: "block", flexShrink: 0, ...style };
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden={rest["aria-label"] ? undefined : true}
      focusable={false}
      style={merged}
      {...rest}
    >
      {ICON_PATHS[name]}
    </svg>
  );
}
