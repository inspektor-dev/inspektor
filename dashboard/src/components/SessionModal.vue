<template>
  <div>
    <div>
      <p><strong>Username:</strong> {{session.postgresUsername}}</p>
    </div>
    <n-divider />
    <div>
      <p><strong>Password:</strong> {{session.postgresPassword}}</p>
    </div>
    <n-divider />
    <p>
      <strong>Note:</strong> Use the following session information to login your
      postgres instance
    </p>
  </div>
</template>

<script>
export default {
  props: ["session"],
  setup: () => {},
};
</script>
