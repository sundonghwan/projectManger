-- 문서 본문을 마크다운 단일 텍스트로 저장. (블록 모델 대체)
ALTER TABLE document ADD COLUMN body TEXT NOT NULL DEFAULT '';
