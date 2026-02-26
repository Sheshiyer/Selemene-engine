interface AccessDeniedProps {
  permission: string;
}

export function AccessDenied({ permission }: AccessDeniedProps) {
  return (
    <section className="panel">
      <h2>Access Denied</h2>
      <p className="helper">
        This account does not currently include the required permission: <code>{permission}</code>
      </p>
      <p className="helper">
        Ask a platform admin to grant access, or enable <code>NEXT_PUBLIC_ADMIN_DEV_MODE=true</code> for
        local scaffold testing.
      </p>
    </section>
  );
}
