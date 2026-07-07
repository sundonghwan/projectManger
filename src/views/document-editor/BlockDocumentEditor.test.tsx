import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { BlockDocumentEditor } from "./BlockDocumentEditor";

describe("BlockDocumentEditor", () => {
  it("renders a markdown-only document instead of crashing", async () => {
    render(
      <BlockDocumentEditor
        documentId="doc-1"
        initialBlocks={null}
        initialMarkdown="# 테스트"
        collaborationState={null}
        onChange={vi.fn()}
        onBlur={vi.fn()}
      />,
    );

    expect(await screen.findByText("테스트")).toBeInTheDocument();
  });
});
