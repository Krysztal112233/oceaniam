export type PageInfo = {
    has_next: boolean;
    total: number;
};

export type PagedResponse<T> = {
    items: T[];
    page_info: PageInfo;
};
