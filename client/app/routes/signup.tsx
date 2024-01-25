import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { schemas } from "~/api/schema";
import { useSignup } from "~/features/auth/use-signup";

const signupSchema = schemas.CreateUser;
type SignupSchema = z.infer<typeof signupSchema>;

export default function Signup() {
  const {
    register,
    handleSubmit: buildHandleSubmit,
    formState: { errors },
  } = useForm<SignupSchema>({
    defaultValues: { name: "", profile: "" },
    resolver: zodResolver(signupSchema),
  });

  const signup = useSignup();
  const handleSubmit = buildHandleSubmit((data) => {
    signup.mutate({ name: data.name, profile: data.profile });
  });

  return (
    <div className="p-5">
      <form className="flex flex-col gap-1" onSubmit={handleSubmit}>
        <div>
          <input
            className="p-1 border rounded w-full"
            placeholder="ユーザー名を入力してください..."
            {...register("name")}
          />
          <p>{errors.name?.message}</p>
        </div>
        <div>
          <textarea
            className="p-1 border rounded w-full"
            placeholder="プロフィールを入力してください..."
            {...register("profile")}
          />
          <p>{errors.profile?.message}</p>
        </div>
        <button className="p-2 bg-neutral-900 text-neutral-200 rounded">
          新規登録
        </button>
      </form>
    </div>
  );
}
