"use client";

import { useRef, useState } from "react";
import { FileText, Image, Upload, X } from "lucide-react";
import type { CollateralDocument } from "@/hooks/useCollateral";

const ACCEPTED_TYPES = ["application/pdf", "image/jpeg", "image/png", "image/webp"];
const MAX_SIZE_MB = 10;

function fileIcon(mimeType?: string) {
  if (mimeType?.startsWith("image/")) return <Image className="h-4 w-4 text-blue-600" />;
  return <FileText className="h-4 w-4 text-gray-600" />;
}

interface DocumentUploadProps {
  documents: CollateralDocument[];
  onChange: (docs: CollateralDocument[]) => void;
}

export function DocumentUpload({ documents, onChange }: DocumentUploadProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [dragOver, setDragOver] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function processFiles(files: FileList | null) {
    if (!files) return;
    setError(null);
    const newDocs: CollateralDocument[] = [];

    for (const file of Array.from(files)) {
      if (!ACCEPTED_TYPES.includes(file.type)) {
        setError(`Unsupported format: ${file.name}. Use PDF or images.`);
        continue;
      }
      if (file.size > MAX_SIZE_MB * 1024 * 1024) {
        setError(`${file.name} exceeds ${MAX_SIZE_MB}MB limit.`);
        continue;
      }
      // Simulate IPFS hash — in production replace with real upload
      const hash = `Qm${btoa(file.name + file.size).slice(0, 20)}`;
      newDocs.push({ name: file.name, hash, mimeType: file.type, size: file.size });
    }

    if (newDocs.length) onChange([...documents, ...newDocs]);
  }

  function remove(hash: string) {
    onChange(documents.filter((d) => d.hash !== hash));
  }

  return (
    <div className="space-y-3">
      <div
        onDragOver={(e) => { e.preventDefault(); setDragOver(true); }}
        onDragLeave={() => setDragOver(false)}
        onDrop={(e) => { e.preventDefault(); setDragOver(false); processFiles(e.dataTransfer.files); }}
        onClick={() => inputRef.current?.click()}
        className={`flex cursor-pointer flex-col items-center justify-center gap-2 rounded-lg border-2 border-dashed p-6 transition ${dragOver ? "border-blue-500 bg-blue-50" : "border-gray-300 hover:border-blue-400"}`}
      >
        <Upload className="h-6 w-6 text-gray-400" />
        <p className="text-sm text-gray-600">
          Drop files here or <span className="font-semibold text-blue-700">browse</span>
        </p>
        <p className="text-xs text-gray-400">PDF, JPEG, PNG, WEBP — max {MAX_SIZE_MB}MB each</p>
        <input
          ref={inputRef}
          type="file"
          multiple
          accept={ACCEPTED_TYPES.join(",")}
          className="hidden"
          onChange={(e) => processFiles(e.target.files)}
        />
      </div>

      {error && <p className="text-xs text-red-600">{error}</p>}

      {documents.length > 0 && (
        <ul className="space-y-2">
          {documents.map((doc) => (
            <li key={doc.hash} className="flex items-center justify-between rounded-lg border border-gray-200 bg-gray-50 px-3 py-2">
              <div className="flex items-center gap-2 min-w-0">
                {fileIcon(doc.mimeType)}
                <span className="truncate text-sm text-gray-800">{doc.name}</span>
                {doc.size && (
                  <span className="shrink-0 text-xs text-gray-400">
                    {(doc.size / 1024).toFixed(0)}KB
                  </span>
                )}
              </div>
              <button
                type="button"
                onClick={(e) => { e.stopPropagation(); remove(doc.hash); }}
                className="ml-2 shrink-0 text-gray-400 hover:text-red-500"
              >
                <X className="h-4 w-4" />
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
