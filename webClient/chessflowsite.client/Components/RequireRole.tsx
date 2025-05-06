import React from 'react';
import { Navigate } from 'react-router-dom';
import {decodeToken } from "../Utils/authToken.ts";


interface RequireRoleProps {
    roles: string[]; // Array of allowed roles
    children: React.ReactNode;
    link?: boolean;
}

function RequireRole({ roles, children, link = false }: RequireRoleProps) {
    const payload = decodeToken();

    const userRoles = payload?.role;

    const hasAccess =
        userRoles &&
        (
            Array.isArray(userRoles)
                ? roles.some(role => userRoles.includes(role)) // Multiple roles
                : roles.includes(userRoles)                    // Single role
        );

    if (!hasAccess) {
        if (link) {
            return <></>;
        }
        else return <Navigate to="/unauthorized" />;
    }

    return <>{children}</>;
}

export default RequireRole;