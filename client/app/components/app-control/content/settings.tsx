import { HandMetalIcon, LogOutIcon, LucideIcon, UserIcon } from "lucide-react";
import { Separator } from "~/components/ui/separator";
import { useAuth } from "~/features/auth/use-auth";

type Props = { width: number };
export const SettingsContent: React.FC<Props> = ({ width }) => {
  const { session, logoutMutation } = useAuth();

  const handleLogout = () => {
    logoutMutation.mutate();
  };

  return (
    <div
      className="flex flex-col space-y-2 p-2 overflow-hidden"
      style={{ width }}
    >
      <div className="flex gap-2 items-start pt-2 px-1">
        <div className="rounded-full border-2">
          <UserIcon />
        </div>
        <div className="flex flex-col min-w-0">
          <p className="text-sm truncate">{session.user.name}</p>
          <p className="text-xs text-muted-foreground">USER</p>
        </div>
      </div>
      <Separator />
      <SettingItem
        icon={LogOutIcon}
        label="ログアウト"
        onClick={handleLogout}
        disabled={logoutMutation.isPending}
      />
      <SettingItem icon={HandMetalIcon} label="設定1" onClick={() => {}} />
      <SettingItem icon={HandMetalIcon} label="設定2" onClick={() => {}} />
      <SettingItem icon={HandMetalIcon} label="設定3" onClick={() => {}} />
    </div>
  );
};

const SettingItem: React.FC<{
  onClick: () => void;
  label: string;
  icon: LucideIcon;
  disabled?: boolean;
}> = ({ label, onClick, icon: Icon, disabled = false }) => {
  return (
    <button
      className="items-center gap-2 whitespace-nowrap flex justify-start text-sm p-2 rounded hover:bg-primary-foreground/10 transition-colors"
      onClick={onClick}
      disabled={disabled}
    >
      <Icon size={20} />
      {label}
    </button>
  );
};
