-- 산출물 파일 업로드 모델: 파일 크기·원본 파일명 컬럼 추가.
-- 버전 히스토리(deliverable_version, current_version)는 비파괴적으로 남겨두되 더 이상 사용하지 않음.
ALTER TABLE deliverable ADD COLUMN file_size    INTEGER;
ALTER TABLE deliverable ADD COLUMN original_name TEXT;
