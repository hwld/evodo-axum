import { ReactNode, createContext, useContext } from "react";
import { Session } from "~/api/types";

type SessionContextData = { session: Session };
const SessionContext = createContext<SessionContextData | undefined>(undefined);

export const SessionProvider: React.FC<{
  children: ReactNode;
  session: Session;
}> = ({ children, session }) => {
  return (
    <SessionContext.Provider value={{ session }}>
      {children}
    </SessionContext.Provider>
  );
};

export const useSession = () => {
  const context = useContext(SessionContext);
  if (!context) {
    throw new Error("SessionProviderが必要です。");
  }

  return context.session;
};
