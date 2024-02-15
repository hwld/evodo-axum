import { motion } from "framer-motion";
import { SearchIcon } from "lucide-react";

export const SearchContent: React.FC = () => {
  return (
    <motion.div layout className="flex gap-2 p-3 items-center w-[500px]">
      <SearchIcon className="shrink-0" size={20} />
      <input
        className="bg-transparent grow placeholder:text-muted-foreground focus-visible:outline-none"
        placeholder="タスクを検索..."
      />
    </motion.div>
  );
};
