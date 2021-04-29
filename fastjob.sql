-- example for storage crate test.
CREATE TABLE `biz_activity`
(
    `id`            varchar(50)  NOT NULL DEFAULT '' COMMENT '唯一活动码',
    `name`          varchar(255) NOT NULL,
    `pc_link`       varchar(255)          DEFAULT NULL,
    `h5_link`       varchar(255)          DEFAULT NULL,
    `sort`          varchar(255) NOT NULL COMMENT '排序',
    `status`        int(11) NOT NULL COMMENT '状态（0：已下线，1：已上线）',
    `version`       int(11) NOT NULL,
    `remark`        varchar(255)          DEFAULT NULL,
    `create_time`   datetime     NOT NULL,
    `delete_flag`   int(1) NOT NULL,
    `pc_banner_img` varchar(255)          DEFAULT NULL,
    `h5_banner_img` varchar(255)          DEFAULT NULL,
    PRIMARY KEY (`id`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8  COMMENT='运营管理-活动管理';


-- 