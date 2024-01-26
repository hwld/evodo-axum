import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { schemas } from "~/api/schema";
import { Button } from "~/components/ui/button";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "~/components/ui/form";
import { Input } from "~/components/ui/input";
import { Node } from "~/components/ui/node";
import { Textarea } from "~/components/ui/textarea";
import { useSignup } from "~/features/auth/use-signup";
import { cn } from "~/lib/utils";
import { CancelSignupButton } from "./cancel-signup-button";

const signupSchema = schemas.CreateUser;
type SignupSchema = z.infer<typeof signupSchema>;

export const SignupFormNode: React.FC = () => {
  const form = useForm<SignupSchema>({
    defaultValues: { name: "", profile: "" },
    resolver: zodResolver(signupSchema),
  });

  const signup = useSignup();
  const handleSubmit = (data: SignupSchema) => {
    signup.mutate({ name: data.name, profile: data.profile });
  };

  return (
    <Node className="w-[400px]">
      <Form {...form}>
        <div className="space-y-5 pt-5 pb-3 w-full">
          <FormField
            control={form.control}
            name="name"
            render={({ field, fieldState }) => {
              return (
                <FormItem>
                  <FormLabel>ユーザー名</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="ユーザー名を入力してください..."
                      autoComplete="off"
                      className={cn(
                        fieldState.error &&
                          "border-destructive focus-visible:ring-destructive"
                      )}
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              );
            }}
          />
          <FormField
            control={form.control}
            name="profile"
            render={({ field }) => {
              return (
                <FormItem>
                  <FormLabel>プロフィール</FormLabel>
                  <FormControl>
                    <Textarea
                      rows={5}
                      placeholder="プロフィールを入力してください..."
                      className="resize-none"
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              );
            }}
          />
          <div className="space-y-2">
            <Button
              className="w-full"
              disabled={signup.isPending}
              // form要素のhandleSubmitを使うと、inputが一つの場合はtextareaがあっても
              // Enterキーでsubmitが送信されてしまうので、それを防ぐためにボタンにトリガーを置く
              onClick={form.handleSubmit(handleSubmit)}
            >
              登録する
            </Button>
            <CancelSignupButton />
          </div>
        </div>
      </Form>
    </Node>
  );
};
