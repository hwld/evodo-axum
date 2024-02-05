import { SearchIcon } from "lucide-react";

export const SearchContent: React.FC = () => {
  return (
    <div className="flex gap-1 p-1 items-center w-[500px]">
      <SearchIcon className="shrink-0" size={20} />
      <input
        className="bg-transparent grow placeholder:text-muted-foreground focus-visible:outline-none"
        placeholder="タスクを検索..."
      />
    </div>
  );
};
